use stead_test_utils::{fixture_path, load_fixture};

#[test]
fn resolves_fixture_path_in_crate() {
    let path = fixture_path("sample.txt");
    assert!(path.ends_with("fixtures/sample.txt"));
}

#[test]
fn loads_fixture_content() {
    let content = load_fixture("sample.txt").expect("fixture should load");
    assert_eq!(content.trim(), "fixture-sample-content");
}

#[test]
fn missing_fixture_returns_error() {
    let err = load_fixture("missing.txt").expect_err("expected missing fixture error");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}
