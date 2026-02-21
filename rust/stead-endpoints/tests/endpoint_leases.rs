use stead_endpoints::{EndpointError, EndpointRegistry};

#[test]
fn claim_new_endpoint_returns_name_owner_assigned_port() {
    let mut registry = EndpointRegistry::with_port_range(4100, 4105);

    let lease = registry.claim("api", "agent-a", Some(4102)).unwrap_claimed();
    assert_eq!(lease.name, "api");
    assert_eq!(lease.owner, "agent-a");
    assert_eq!(lease.port, 4102);
    assert_eq!(lease.url(), "http://api.localhost:4102");
}

#[test]
fn reclaim_same_owner_is_idempotent() {
    let mut registry = EndpointRegistry::with_port_range(4100, 4105);

    let first = registry.claim("api", "agent-a", Some(4102)).unwrap_claimed();
    let second = registry.claim("api", "agent-a", Some(4104)).unwrap_claimed();

    assert_eq!(first, second);
    assert_eq!(second.port, 4102);
}

#[test]
fn release_requires_owner() {
    let mut registry = EndpointRegistry::with_port_range(4100, 4105);
    registry.claim("api", "agent-a", Some(4102));

    let err = registry
        .release("api", "agent-b")
        .expect_err("non-owner release must fail");

    assert_eq!(
        err,
        EndpointError::NotOwner {
            name: "api".to_string(),
            expected_owner: "agent-a".to_string(),
            attempted_by: "agent-b".to_string(),
        }
    );

    let released = registry.release("api", "agent-a").unwrap();
    assert_eq!(released.port, 4102);
}

#[test]
fn export_import_round_trip_preserves_state() {
    let mut source = EndpointRegistry::with_port_range(4100, 4105);
    source.claim("api", "agent-a", Some(4101));
    source.claim("dashboard", "agent-b", Some(4103));

    let exported = source.export_leases();

    let mut restored = EndpointRegistry::with_port_range(4100, 4105);
    restored.import_leases(exported);

    let mut names = restored
        .list()
        .into_iter()
        .map(|lease| lease.name)
        .collect::<Vec<_>>();
    names.sort();

    assert_eq!(names, vec!["api".to_string(), "dashboard".to_string()]);
}
