#[test]
fn ci_workflow_contains_benchmark_job() {
    let workflow = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(".github")
            .join("workflows")
            .join("ci.yml"),
    )
    .expect("workflow should exist");

    assert!(workflow.contains("benchmark:"), "benchmark job missing");
}

#[test]
fn ci_workflow_contains_coverage_gate_job() {
    let workflow = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(".github")
            .join("workflows")
            .join("ci.yml"),
    )
    .expect("workflow should exist");

    assert!(workflow.contains("coverage:"), "coverage job missing");
    assert!(
        workflow.contains("cargo-llvm-cov"),
        "coverage tool install missing"
    );
}

#[test]
fn ci_workflow_enforces_90_percent_domain_coverage_thresholds() {
    let workflow = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(".github")
            .join("workflows")
            .join("ci.yml"),
    )
    .expect("workflow should exist");

    assert!(
        workflow.contains("-p stead-contracts --fail-under-lines 90"),
        "stead-contracts threshold missing"
    );
    assert!(
        workflow.contains("-p stead-resources --fail-under-lines 90"),
        "stead-resources threshold missing"
    );
    assert!(
        workflow.contains("-p stead-usf --fail-under-lines 90"),
        "stead-usf threshold missing"
    );
}
