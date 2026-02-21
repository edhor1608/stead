use std::time::Duration;
use stead_daemon::{ApiRequest, ApiResponse, Daemon, DaemonEventKind};
use stead_resources::{ClaimResult, ResourceKey};
use tempfile::tempdir;

#[test]
fn two_agents_contending_for_same_port_get_deterministic_negotiation() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::new(&db).unwrap();

    let first = daemon
        .handle(ApiRequest::ClaimResource {
            resource: ResourceKey::port(3000),
            owner: "agent-a".to_string(),
        })
        .unwrap();

    match first.data {
        ApiResponse::ResourceClaim(result) => {
            assert!(matches!(result, ClaimResult::Claimed(_)));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let second = daemon
        .handle(ApiRequest::ClaimResource {
            resource: ResourceKey::port(3000),
            owner: "agent-b".to_string(),
        })
        .unwrap();

    match second.data {
        ApiResponse::ResourceClaim(ClaimResult::Negotiated { assigned, .. }) => {
            assert_eq!(assigned.resource, ResourceKey::port(3001));
            assert_eq!(assigned.owner, "agent-b");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn unresolved_conflict_emits_escalation_event() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::with_port_range(&db, 3000, 3000).unwrap();
    let stream = daemon.subscribe();

    daemon
        .handle(ApiRequest::ClaimResource {
            resource: ResourceKey::port(3000),
            owner: "agent-a".to_string(),
        })
        .unwrap();

    let second = daemon
        .handle(ApiRequest::ClaimResource {
            resource: ResourceKey::port(3000),
            owner: "agent-b".to_string(),
        })
        .unwrap();

    match second.data {
        ApiResponse::ResourceClaim(ClaimResult::Conflict(_)) => {}
        other => panic!("unexpected response: {other:?}"),
    }

    let escalation = stream
        .recv_timeout(Duration::from_secs(1))
        .expect("expected escalation event");

    match escalation.kind {
        DaemonEventKind::ResourceConflictEscalated {
            resource,
            requested_by,
            held_by,
            reason,
        } => {
            assert_eq!(resource, ResourceKey::port(3000));
            assert_eq!(requested_by, "agent-b");
            assert_eq!(held_by, "agent-a");
            assert_eq!(reason, "port_range_exhausted");
        }
        other => panic!("unexpected event: {other:?}"),
    }
}
