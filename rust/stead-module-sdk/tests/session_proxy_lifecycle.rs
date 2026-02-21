use stead_module_sdk::{SessionProxy, SessionProxyError};

#[test]
fn identities_are_unique_per_creation_and_project() {
    let mut proxy = SessionProxy::default();

    let one = proxy.create_identity("project-a");
    let two = proxy.create_identity("project-a");
    let three = proxy.create_identity("project-b");

    assert_ne!(one, two);
    assert_ne!(one, three);
    assert_ne!(two, three);
}

#[test]
fn destroying_identity_invalidates_only_that_identity() {
    let mut proxy = SessionProxy::default();

    let a = proxy.create_identity("project-a");
    let b = proxy.create_identity("project-a");

    let token_a = proxy.issue_token("project-a", &a).unwrap();
    let token_b = proxy.issue_token("project-a", &b).unwrap();

    proxy.destroy_identity("project-a", &a);

    let err = proxy
        .validate_token("project-a", &token_a)
        .expect_err("destroyed identity must fail validation");
    assert_eq!(err, SessionProxyError::UnknownIdentity);

    let still_valid = proxy.validate_token("project-a", &token_b).unwrap();
    assert_eq!(still_valid, b);
}
