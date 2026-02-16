use std::path::Path;

fn crate_manifest(crate_name: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::read_to_string(root.join(crate_name).join("Cargo.toml"))
        .expect("crate Cargo.toml should exist")
}

#[test]
fn reusable_crates_have_exportable_package_metadata() {
    let reusable_crates = [
        "stead-contracts",
        "stead-usf",
        "stead-resources",
        "stead-endpoints",
        "stead-module-sdk",
    ];

    for name in reusable_crates {
        let manifest = crate_manifest(name);
        assert!(
            manifest.contains("description = "),
            "{name}: missing description"
        );
        assert!(
            manifest.contains("license = \"MIT\""),
            "{name}: missing MIT license"
        );
        assert!(
            manifest.contains("repository = \"https://github.com/edhor1608/stead\""),
            "{name}: missing repository"
        );
        assert!(
            manifest.contains("readme = \"README.md\""),
            "{name}: missing readme path"
        );
    }
}
