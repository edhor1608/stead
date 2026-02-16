use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).expect("file should exist")
}

#[test]
fn root_readme_tracks_grouped_cli_surface() {
    let readme = read_repo_file("README.md");

    for family in [
        "stead contract",
        "stead session",
        "stead resource",
        "stead attention",
        "stead context",
        "stead module",
        "stead daemon",
    ] {
        assert!(
            readme.contains(family),
            "README should document grouped family: {family}"
        );
    }

    for legacy in ["stead run", "stead list", "stead show", "stead verify"] {
        assert!(
            !readme.contains(legacy),
            "README should not claim legacy top-level command: {legacy}"
        );
    }
}

#[test]
fn rust_readme_tracks_daemon_backed_workspace_structure() {
    let readme = read_repo_file("rust/README.md");

    for crate_name in [
        "stead-daemon/",
        "stead-contracts/",
        "stead-resources/",
        "stead-endpoints/",
        "stead-usf/",
        "stead-module-sdk/",
    ] {
        assert!(
            readme.contains(crate_name),
            "rust/README.md missing crate description: {crate_name}"
        );
    }

    assert!(
        readme.contains("daemon-backed"),
        "rust/README.md should describe daemon-backed CLI behavior"
    );
    assert!(
        !readme.contains("stead-core/             # Library — all logic lives here"),
        "rust/README.md still claims a stale monolith layout"
    );
}

#[test]
fn planning_baseline_tracks_daemon_backed_runtime_claims() {
    let baseline = read_repo_file("docs/plans/planning-baseline-2026-02-13.md");

    assert!(
        baseline.contains("daemon-backed"),
        "planning baseline should acknowledge daemon-backed CLI runtime"
    );
    assert!(
        !baseline.contains("no HTTP API and no daemon requirement"),
        "planning baseline still contains obsolete monolith statement"
    );
}

#[test]
fn canonical_decisions_do_not_reintroduce_legacy_top_level_commands() {
    let canonical = read_repo_file("docs/plans/canonical-decisions-2026-02-11.md");

    assert!(
        canonical.contains("grouped command families"),
        "canonical decisions should describe grouped command families"
    );
    assert!(
        !canonical.contains("run`, `list`, `show"),
        "canonical decisions still reference legacy top-level command set"
    );
}
