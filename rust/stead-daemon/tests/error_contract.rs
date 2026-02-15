use serde_json::Value;
use stead_contracts::ContractStatus;
use stead_daemon::{ApiRequest, Daemon};

#[test]
fn returns_typed_error_for_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("daemon.db");
    let daemon = Daemon::new(&db_path).unwrap();

    let err = daemon
        .handle(ApiRequest::GetContract {
            id: "missing".into(),
        })
        .expect_err("missing contract should return error");

    assert_eq!(err.code, "not_found");
}

#[test]
fn returns_typed_error_for_invalid_transition() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("daemon.db");
    let daemon = Daemon::new(&db_path).unwrap();

    daemon
        .handle(ApiRequest::CreateContract {
            id: "bad-transition".into(),
            blocked_by: vec![],
        })
        .unwrap();

    let err = daemon
        .handle(ApiRequest::TransitionContract {
            id: "bad-transition".into(),
            to: ContractStatus::Completed,
        })
        .expect_err("ready -> completed should be invalid");

    assert_eq!(err.code, "invalid_transition");
}

#[test]
fn error_json_shape_is_stable() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("daemon.db");
    let daemon = Daemon::new(&db_path).unwrap();

    let err = daemon
        .handle(ApiRequest::GetContract { id: "x".into() })
        .expect_err("should fail");

    let json = serde_json::to_string(&err).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();

    assert!(value.get("code").is_some());
    assert!(value.get("message").is_some());
}
