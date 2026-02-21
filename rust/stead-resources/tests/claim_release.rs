use stead_resources::{ClaimResult, ResourceKey, ResourceRegistry};

#[test]
fn claiming_free_resource_creates_owned_lease() {
    let mut registry = ResourceRegistry::default();

    let result = registry.claim(ResourceKey::port(3000), "agent-a");

    match result {
        ClaimResult::Claimed(lease) => {
            assert_eq!(lease.resource, ResourceKey::port(3000));
            assert_eq!(lease.owner, "agent-a");
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn owner_can_release_and_other_agent_can_claim_afterwards() {
    let mut registry = ResourceRegistry::default();

    let first = registry.claim(ResourceKey::port(3000), "agent-a");
    assert!(matches!(first, ClaimResult::Claimed(_)));

    registry
        .release(ResourceKey::port(3000), "agent-a")
        .expect("owner should be able to release lease");

    let second = registry.claim(ResourceKey::port(3000), "agent-b");
    assert!(matches!(second, ClaimResult::Claimed(_)));
}

#[test]
fn non_owner_cannot_release_lease() {
    let mut registry = ResourceRegistry::default();

    let first = registry.claim(ResourceKey::port(3000), "agent-a");
    assert!(matches!(first, ClaimResult::Claimed(_)));

    let error = registry
        .release(ResourceKey::port(3000), "agent-b")
        .expect_err("non-owner release should fail");

    assert_eq!(error.code(), "not_owner");
}
