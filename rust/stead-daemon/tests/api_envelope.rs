use stead_daemon::{API_VERSION, ApiRequest, ApiResponse, Daemon};

#[test]
fn responses_use_versioned_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("daemon.db");
    let daemon = Daemon::new(&db_path).unwrap();

    let envelope = daemon.handle(ApiRequest::Health).unwrap();
    assert_eq!(envelope.version, API_VERSION);
    assert_eq!(
        envelope.data,
        ApiResponse::Health {
            status: "ok".into()
        }
    );
}
