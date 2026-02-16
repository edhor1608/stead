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
fn rust_workspace_members_exclude_legacy_stead_core() {
    let workspace_manifest = read_repo_file("rust/Cargo.toml");
    assert!(
        !workspace_manifest.contains("stead-core"),
        "rust workspace still includes legacy stead-core member"
    );
}

#[test]
fn shipped_readmes_do_not_advertise_stead_core() {
    let root_readme = read_repo_file("README.md");
    let rust_readme = read_repo_file("rust/README.md");

    assert!(
        !root_readme.contains("stead-core/"),
        "top-level README should not advertise stead-core in rewrite surface"
    );
    assert!(
        !rust_readme.contains("stead-core/"),
        "rust README should not advertise stead-core in rewrite surface"
    );
}
