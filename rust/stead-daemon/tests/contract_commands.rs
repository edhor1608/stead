use stead_contracts::ContractStatus;
use stead_daemon::{ApiRequest, ApiResponse, Daemon};

#[test]
fn create_transition_and_get_contract_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("daemon.db");
    let daemon = Daemon::new(&db_path).unwrap();

    let created = daemon
        .handle(ApiRequest::CreateContract {
            id: "m3-c1".into(),
            blocked_by: vec![],
        })
        .unwrap();

    match created.data {
        ApiResponse::ContractState(contract) => {
            assert_eq!(contract.id, "m3-c1");
            assert_eq!(contract.status, ContractStatus::Ready);
        }
        _ => panic!("expected contract response"),
    }

    let transitioned = daemon
        .handle(ApiRequest::TransitionContract {
            id: "m3-c1".into(),
            to: ContractStatus::Claimed,
        })
        .unwrap();

    match transitioned.data {
        ApiResponse::ContractState(contract) => {
            assert_eq!(contract.status, ContractStatus::Claimed);
        }
        _ => panic!("expected contract response"),
    }

    let fetched = daemon
        .handle(ApiRequest::GetContract { id: "m3-c1".into() })
        .unwrap();

    match fetched.data {
        ApiResponse::ContractState(contract) => {
            assert_eq!(contract.id, "m3-c1");
            assert_eq!(contract.status, ContractStatus::Claimed);
        }
        _ => panic!("expected contract response"),
    }
}
