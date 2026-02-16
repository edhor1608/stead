use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use stead_contracts::ContractStatus;
use stead_daemon::{ApiRequest, ApiResponse, Daemon};
use tempfile::tempdir;

#[test]
fn state_propagation_updates_attention_within_5_seconds() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Daemon::new(&db).unwrap();

    daemon
        .handle(ApiRequest::CreateContract {
            id: "latency-c1".into(),
            blocked_by: vec![],
        })
        .unwrap();

    daemon
        .handle(ApiRequest::TransitionContract {
            id: "latency-c1".into(),
            to: ContractStatus::Claimed,
        })
        .unwrap();
    daemon
        .handle(ApiRequest::TransitionContract {
            id: "latency-c1".into(),
            to: ContractStatus::Executing,
        })
        .unwrap();

    let start = Instant::now();
    let counts = loop {
        let response = daemon.handle(ApiRequest::AttentionStatus).unwrap();
        let counts = match response.data {
            ApiResponse::Attention(counts) => counts,
            other => panic!("unexpected response: {other:?}"),
        };

        if counts.running >= 1 {
            break counts;
        }

        assert!(
            start.elapsed() < Duration::from_secs(5),
            "state propagation exceeded 5 seconds"
        );
        thread::sleep(Duration::from_millis(20));
    };

    assert!(counts.running >= 1);
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn concurrent_client_soak_completes_without_failures() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("stead.db");
    let daemon = Arc::new(Daemon::new(&db).unwrap());

    const WORKERS: usize = 6;
    const OPS_PER_WORKER: usize = 60;

    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..WORKERS {
        let daemon = daemon.clone();
        handles.push(thread::spawn(move || {
            for op in 0..OPS_PER_WORKER {
                let id = format!("soak-{worker}-{op}");
                daemon
                    .handle(ApiRequest::CreateContract {
                        id,
                        blocked_by: vec![],
                    })
                    .unwrap();
                daemon.handle(ApiRequest::ListContracts).unwrap();
                daemon.handle(ApiRequest::AttentionStatus).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let listed = daemon.handle(ApiRequest::ListContracts).unwrap();
    match listed.data {
        ApiResponse::Contracts(contracts) => {
            assert_eq!(contracts.len(), WORKERS * OPS_PER_WORKER);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    assert!(
        start.elapsed() < Duration::from_secs(30),
        "soak test exceeded 30 seconds"
    );
}
