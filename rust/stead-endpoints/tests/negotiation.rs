use stead_endpoints::{EndpointClaimResult, EndpointEvent, EndpointRegistry};

#[test]
fn conflict_resolves_to_lowest_available_next_port_in_range() {
    let mut registry = EndpointRegistry::with_port_range(4100, 4103);

    registry.claim("alpha", "agent-a", Some(4101));
    registry.claim("bravo", "agent-b", Some(4102));

    let third = registry.claim("charlie", "agent-c", Some(4101));

    match third {
        EndpointClaimResult::Negotiated {
            requested_port,
            assigned,
            ..
        } => {
            assert_eq!(requested_port, 4101);
            assert_eq!(assigned.port, 4103);
            assert_eq!(assigned.name, "charlie");
        }
        other => panic!("expected negotiated claim, got {other:?}"),
    }
}

#[test]
fn exhausted_range_emits_escalation_event() {
    let mut registry = EndpointRegistry::with_port_range(4100, 4101);

    registry.claim("alpha", "agent-a", Some(4100));
    registry.claim("bravo", "agent-b", Some(4101));

    let exhausted = registry.claim("charlie", "agent-c", Some(4100));
    assert!(matches!(exhausted, EndpointClaimResult::Conflict(_)));

    let events = registry.drain_events();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        EndpointEvent::RangeExhausted {
            name: "charlie".to_string(),
            owner: "agent-c".to_string(),
            requested_port: 4100,
            reason: "endpoint_range_exhausted",
        }
    );
}

#[test]
fn negotiation_is_deterministic_for_ordered_claim_sequence() {
    let mut registry = EndpointRegistry::with_port_range(4100, 4104);

    registry.claim("alpha", "agent-a", Some(4100));
    registry.claim("bravo", "agent-b", Some(4101));

    let c1 = registry.claim("charlie", "agent-c", Some(4100));
    let c2 = registry.claim("delta", "agent-d", Some(4100));

    let assigned = [c1, c2]
        .into_iter()
        .map(|result| match result {
            EndpointClaimResult::Negotiated { assigned, .. } => assigned.port,
            other => panic!("expected negotiated result, got {other:?}"),
        })
        .collect::<Vec<_>>();

    assert_eq!(assigned, vec![4102, 4103]);
}
