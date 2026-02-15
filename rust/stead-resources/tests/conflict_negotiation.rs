use stead_resources::{ClaimResult, ResourceKey, ResourceRegistry};

#[test]
fn conflicting_port_claim_negotiates_next_free_port() {
    let mut registry = ResourceRegistry::with_port_range(3000, 3003);

    assert!(matches!(
        registry.claim(ResourceKey::port(3000), "agent-a"),
        ClaimResult::Claimed(_)
    ));

    let result = registry.claim(ResourceKey::port(3000), "agent-b");

    match result {
        ClaimResult::Negotiated {
            requested,
            assigned,
            held_by,
        } => {
            assert_eq!(requested, ResourceKey::port(3000));
            assert_eq!(held_by.owner, "agent-a");
            assert_eq!(assigned.resource, ResourceKey::port(3001));
            assert_eq!(assigned.owner, "agent-b");
        }
        other => panic!("expected negotiated claim, got {other:?}"),
    }
}

#[test]
fn negotiation_is_deterministic_and_selects_lowest_available_port() {
    let mut registry = ResourceRegistry::with_port_range(3000, 3005);

    assert!(matches!(
        registry.claim(ResourceKey::port(3000), "agent-a"),
        ClaimResult::Claimed(_)
    ));
    assert!(matches!(
        registry.claim(ResourceKey::port(3001), "agent-z"),
        ClaimResult::Claimed(_)
    ));
    assert!(matches!(
        registry.claim(ResourceKey::port(3002), "agent-y"),
        ClaimResult::Claimed(_)
    ));

    let result = registry.claim(ResourceKey::port(3000), "agent-b");

    match result {
        ClaimResult::Negotiated { assigned, .. } => {
            assert_eq!(assigned.resource, ResourceKey::port(3003));
        }
        other => panic!("expected negotiated claim, got {other:?}"),
    }
}

#[test]
fn conflict_is_explicit_when_port_range_exhausted() {
    let mut registry = ResourceRegistry::with_port_range(3000, 3001);

    assert!(matches!(
        registry.claim(ResourceKey::port(3000), "agent-a"),
        ClaimResult::Claimed(_)
    ));
    assert!(matches!(
        registry.claim(ResourceKey::port(3001), "agent-c"),
        ClaimResult::Claimed(_)
    ));

    let result = registry.claim(ResourceKey::port(3000), "agent-b");

    match result {
        ClaimResult::Conflict(conflict) => {
            assert_eq!(conflict.requested, ResourceKey::port(3000));
            assert_eq!(conflict.held_by.owner, "agent-a");
        }
        other => panic!("expected conflict, got {other:?}"),
    }
}
