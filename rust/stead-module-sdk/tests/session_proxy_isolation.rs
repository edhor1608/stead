use stead_module_sdk::{SessionProxy, SessionProxyError};

#[test]
fn token_is_valid_only_within_its_project_boundary() {
    let mut proxy = SessionProxy::default();
    let identity = proxy.create_identity("project-a");
    let token = proxy.issue_token("project-a", &identity).unwrap();

    let validated = proxy.validate_token("project-a", &token).unwrap();
    assert_eq!(validated, identity);

    let err = proxy
        .validate_token("project-b", &token)
        .expect_err("cross-project token use must fail");

    assert_eq!(err, SessionProxyError::ProjectIsolationViolation);
}
