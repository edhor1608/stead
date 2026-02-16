use std::time::Duration;

use stead_daemon::{ApiRequest, Daemon, DaemonEventKind};

#[test]
fn endpoint_range_exhaustion_is_published_to_subscribers() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::with_port_range(&db, 4100, 4100).unwrap();
    let stream = daemon.subscribe();

    daemon
        .handle(ApiRequest::ClaimEndpoint {
            name: "api".to_string(),
            owner: "agent-a".to_string(),
            port: Some(4100),
        })
        .unwrap();

    let err = daemon
        .handle(ApiRequest::ClaimEndpoint {
            name: "web".to_string(),
            owner: "agent-b".to_string(),
            port: Some(4100),
        })
        .expect_err("second endpoint claim must exhaust configured range");

    assert_eq!(err.code, "endpoint_range_exhausted");

    let event = stream
        .recv_timeout(Duration::from_secs(1))
        .expect("expected endpoint escalation event");

    match event.kind {
        DaemonEventKind::EndpointRangeExhausted {
            name,
            owner,
            requested_port,
            reason,
        } => {
            assert_eq!(name, "web");
            assert_eq!(owner, "agent-b");
            assert_eq!(requested_port, 4100);
            assert_eq!(reason, "endpoint_range_exhausted");
        }
        other => panic!("unexpected daemon event: {other:?}"),
    }
}

#[test]
fn endpoint_events_are_replayable_by_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::with_port_range(&db, 4100, 4100).unwrap();

    daemon
        .handle(ApiRequest::ClaimEndpoint {
            name: "api".to_string(),
            owner: "agent-a".to_string(),
            port: Some(4100),
        })
        .unwrap();

    let _ = daemon.handle(ApiRequest::ClaimEndpoint {
        name: "web".to_string(),
        owner: "agent-b".to_string(),
        port: Some(4100),
    });

    let events = daemon.replay_from(0);
    assert!(events.iter().any(|event| {
        matches!(
            event.kind,
            DaemonEventKind::EndpointRangeExhausted {
                ref name,
                ref owner,
                requested_port: 4100,
                reason: "endpoint_range_exhausted",
            } if name == "web" && owner == "agent-b"
        )
    }));
}
