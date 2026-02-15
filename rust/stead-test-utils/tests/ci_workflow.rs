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
