//! Run command - create and execute a contract

use crate::cli::RunEngine;
use crate::schema::Contract;
use crate::storage::{self, Storage};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Execute the run command
pub fn execute(task: &str, verify_cmd: &str, engine: RunEngine, json_output: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let db = storage::sqlite::open_default(&cwd)?;
    execute_with_storage(task, verify_cmd, engine, json_output, &cwd, &db)
}

/// Execute with explicit working directory (for testing)
pub fn execute_with_cwd(
    task: &str,
    verify_cmd: &str,
    engine: RunEngine,
    json_output: bool,
    cwd: &Path,
) -> Result<()> {
    let db = storage::sqlite::open_default(cwd)?;
    execute_with_storage(task, verify_cmd, engine, json_output, cwd, &db)
}

/// Execute with a specific storage backend
pub fn execute_with_storage(
    task: &str,
    verify_cmd: &str,
    engine: RunEngine,
    json_output: bool,
    cwd: &Path,
    storage: &dyn Storage,
) -> Result<()> {
    // Create contract (Pending)
    let mut contract = Contract::new(task, verify_cmd);
    contract.project_path = cwd.to_string_lossy().to_string();
    storage.save_contract(&contract)?;

    if !json_output {
        println!("Contract created: {}", contract.id);
    }

    // Pending → Ready → Claimed → Executing
    contract.mark_ready().expect("pending -> ready");
    contract.claim("stead-cli").expect("ready -> claimed");
    contract.start().expect("claimed -> executing");
    storage.update_contract(&contract)?;

    if !json_output {
        println!("Executing task...");
    }

    // Execute the selected engine with the task (best-effort; verification decides PASS/FAIL)
    let engine_result = spawn_engine(engine, task, cwd);
    let engine_error = match &engine_result {
        Ok(()) => None,
        Err(e) => {
            if !json_output {
                eprintln!("Warning: Execution failed: {}", e);
            }
            Some(format!("[Engine failed: {}]", e))
        }
    };

    // Executing → Verifying
    contract.begin_verify().expect("executing -> verifying");
    storage.update_contract(&contract)?;

    if !json_output {
        println!("Running verification...");
    }

    // Run verification
    let (passed, output) = run_verification(verify_cmd)?;

    // Combine engine error with verification output
    let combined_output = match (engine_error, output) {
        (Some(err), Some(out)) => Some(format!("{}\n{}", err, out)),
        (Some(err), None) => Some(err),
        (None, out) => out,
    };

    // Verifying → Completed/Failed
    contract.complete(passed, combined_output);
    storage.update_contract(&contract)?;

    if json_output {
        println!("{}", serde_json::to_string(&contract)?);
    } else {
        println!(
            "Contract {}: {}",
            contract.id,
            if passed { "PASSED" } else { "FAILED" }
        );
        if let Some(ref out) = contract.output {
            if !out.is_empty() {
                println!("\nOutput:\n{}", out);
            }
        }
    }

    Ok(())
}

fn spawn_engine(engine: RunEngine, task: &str, cwd: &Path) -> Result<()> {
    if let RunEngine::None = engine {
        return Ok(());
    }

    let mut cmd = match engine {
        RunEngine::Claude => {
            let mut c = Command::new("claude");
            c.args(["-p", task]);
            c
        }
        RunEngine::Codex => {
            let mut c = Command::new("codex");
            // Non-interactive run; keep it repo-scoped.
            c.args(["exec", "-C"]).arg(cwd).arg(task);
            c
        }
        RunEngine::OpenCode => {
            let mut c = Command::new("opencode");
            c.args(["run", task]);
            c
        }
        RunEngine::None => unreachable!(),
    };

    cmd.current_dir(cwd);
    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute engine: {:?}", engine))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = [stdout.trim(), stderr.trim()]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if combined.is_empty() {
            anyhow::bail!("Engine exited with status {}", output.status);
        } else {
            anyhow::bail!("Engine exited with status {}: {}", output.status, combined);
        }
    }

    Ok(())
}

/// Run verification command and capture output
fn run_verification(cmd: &str) -> Result<(bool, Option<String>)> {
    let (shell, flag) = if cfg!(target_os = "windows") {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };

    let output = Command::new(shell)
        .args([flag, cmd])
        .output()
        .context("Failed to run verification command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = [stdout.trim(), stderr.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let output_str = if combined.is_empty() {
        None
    } else {
        Some(combined)
    };

    Ok((output.status.success(), output_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    #[cfg(unix)]
    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[cfg(unix)]
    fn make_temp_dir() -> std::path::PathBuf {
        let unique = format!(
            "stead-run-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn test_verification_pass() {
        let (passed, output) = run_verification("echo hello").unwrap();
        assert!(passed);
        assert_eq!(output, Some("hello".to_string()));
    }

    #[test]
    fn test_verification_fail() {
        let cmd = if cfg!(target_os = "windows") {
            "exit 1"
        } else {
            "false"
        };
        let (passed, _) = run_verification(cmd).unwrap();
        assert!(!passed);
    }

    #[test]
    fn test_verification_captures_stderr() {
        let cmd = if cfg!(target_os = "windows") {
            "echo error 1>&2"
        } else {
            "echo error >&2"
        };
        let (passed, output) = run_verification(cmd).unwrap();
        assert!(passed);
        assert!(output.unwrap().contains("error"));
    }

    #[cfg(unix)]
    #[test]
    fn test_spawn_engine_error_includes_status_stdout_and_stderr() {
        use std::os::unix::fs::PermissionsExt;

        struct Cleanup {
            tmp: std::path::PathBuf,
            old_path: String,
        }

        impl Drop for Cleanup {
            fn drop(&mut self) {
                std::env::set_var("PATH", &self.old_path);
                let _ = std::fs::remove_dir_all(&self.tmp);
            }
        }

        let _guard = test_lock().lock().expect("lock");
        let tmp = make_temp_dir();
        let fake = tmp.join("codex");

        std::fs::write(
            &fake,
            "#!/bin/sh\necho stdout-msg\necho stderr-msg 1>&2\nexit 7\n",
        )
        .expect("write fake codex");

        let mut perms = std::fs::metadata(&fake).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&fake, perms).expect("chmod");

        let old_path = std::env::var("PATH").unwrap_or_default();
        let _cleanup = Cleanup {
            tmp: tmp.clone(),
            old_path: old_path.clone(),
        };
        std::env::set_var("PATH", format!("{}:{}", tmp.display(), old_path));

        let err = spawn_engine(RunEngine::Codex, "demo task", &tmp).expect_err("should fail");

        let message = format!("{:#}", err);
        assert!(
            message.contains("stdout-msg"),
            "error should include stdout: {message}"
        );
        assert!(
            message.contains("stderr-msg"),
            "error should include stderr: {message}"
        );
        assert!(
            message.contains("exit status")
                || message.contains("status")
                || message.contains("code"),
            "error should include process status: {message}"
        );
    }
}
