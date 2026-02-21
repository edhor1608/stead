use stead_contracts::ContractStatus;
use stead_daemon::{ApiRequest, Daemon, DaemonEventKind};

#[test]
fn supports_subscribe_and_replay_by_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("daemon.db");
    let daemon = Daemon::new(&db_path).unwrap();

    let rx = daemon.subscribe();

    daemon
        .handle(ApiRequest::CreateContract {
            id: "evt-c1".into(),
            blocked_by: vec![],
        })
        .unwrap();

    daemon
        .handle(ApiRequest::TransitionContract {
            id: "evt-c1".into(),
            to: ContractStatus::Claimed,
        })
        .unwrap();

    let first = rx.recv().unwrap();
    let second = rx.recv().unwrap();

    assert_eq!(first.cursor, 1);
    assert_eq!(second.cursor, 2);
    assert!(matches!(
        first.kind,
        DaemonEventKind::ContractCreated { .. }
    ));
    assert!(matches!(
        second.kind,
        DaemonEventKind::ContractTransitioned { .. }
    ));

    let replay = daemon.replay_from(1);
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].cursor, 2);
}
