use stead_daemon::{ApiRequest, ApiResponse, Daemon};
use tempfile::tempdir;

#[test]
fn lists_contracts_via_daemon_api() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::new(&db).unwrap();

    daemon
        .handle(ApiRequest::CreateContract {
            id: "c-1".into(),
            blocked_by: vec![],
        })
        .unwrap();

    let response = daemon.handle(ApiRequest::ListContracts).unwrap();

    match response.data {
        ApiResponse::Contracts(contracts) => {
            assert_eq!(contracts.len(), 1);
            assert_eq!(contracts[0].id, "c-1");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn reports_attention_counts_via_daemon_api() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::new(&db).unwrap();

    daemon
        .handle(ApiRequest::CreateContract {
            id: "c-queued".into(),
            blocked_by: vec!["dep".into()],
        })
        .unwrap();

    let response = daemon.handle(ApiRequest::AttentionStatus).unwrap();

    match response.data {
        ApiResponse::Attention(counts) => {
            assert_eq!(counts.queued, 1);
            assert_eq!(counts.running, 0);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
