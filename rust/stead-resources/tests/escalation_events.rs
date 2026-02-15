use stead_resources::{ClaimResult, ResourceEvent, ResourceKey, ResourceRegistry};

#[test]
fn emits_conflict_escalation_event_when_negotiation_fails() {
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
    assert!(matches!(result, ClaimResult::Conflict(_)));

    let events = registry.drain_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        ResourceEvent::ConflictEscalated {
            requested,
            requested_by,
            held_by,
            reason,
        } => {
            assert_eq!(*requested, ResourceKey::port(3000));
            assert_eq!(requested_by, "agent-b");
            assert_eq!(held_by, "agent-a");
            assert_eq!(*reason, "port_range_exhausted");
        }
    }
}

#[test]
fn silent_negotiation_does_not_emit_escalation_event() {
    let mut registry = ResourceRegistry::with_port_range(3000, 3003);

    assert!(matches!(
        registry.claim(ResourceKey::port(3000), "agent-a"),
        ClaimResult::Claimed(_)
    ));

    let result = registry.claim(ResourceKey::port(3000), "agent-b");
    assert!(matches!(result, ClaimResult::Negotiated { .. }));

    let events = registry.drain_events();
    assert!(events.is_empty());
}
