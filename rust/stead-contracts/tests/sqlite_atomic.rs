use stead_contracts::{Contract, ContractEvent, ContractStatus, SqliteContractStore};

#[test]
fn transition_write_is_atomic_snapshot_plus_event() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let mut contract = Contract::new("c-atomic", vec![]);
    store.save_contract(&contract).unwrap();

    let event = contract.transition_to(ContractStatus::Claimed).unwrap();
    store.record_transition(&contract, &event).unwrap();

    let loaded = store.load_contract("c-atomic").unwrap().unwrap();
    assert_eq!(loaded.status, ContractStatus::Claimed);

    let events = store.list_events("c-atomic").unwrap();
    assert_eq!(events, vec![event]);
}

#[test]
fn failed_event_insert_rolls_back_snapshot_update() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let mut contract = Contract::new("c-rollback", vec![]);
    store.save_contract(&contract).unwrap();

    let _valid_event = contract.transition_to(ContractStatus::Claimed).unwrap();
    let invalid_event = ContractEvent {
        contract_id: "missing-contract".to_string(),
        from: ContractStatus::Ready,
        to: ContractStatus::Claimed,
    };

    let result = store.record_transition(&contract, &invalid_event);
    assert!(
        result.is_err(),
        "insert should fail due to foreign key constraint"
    );

    let loaded = store.load_contract("c-rollback").unwrap().unwrap();
    assert_eq!(
        loaded.status,
        ContractStatus::Ready,
        "snapshot update should have rolled back"
    );
    assert!(store.list_events("c-rollback").unwrap().is_empty());
}

#[test]
fn rejects_mismatched_contract_and_event_ids_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let mut contract_a = Contract::new("c-a", vec![]);
    let mut contract_b = Contract::new("c-b", vec![]);
    store.save_contract(&contract_a).unwrap();
    store.save_contract(&contract_b).unwrap();

    let _a_event = contract_a.transition_to(ContractStatus::Claimed).unwrap();
    let b_event = contract_b.transition_to(ContractStatus::Claimed).unwrap();

    let result = store.record_transition(&contract_a, &b_event);
    assert!(
        result.is_err(),
        "mismatched contract/event ids must be rejected"
    );

    let loaded_a = store.load_contract("c-a").unwrap().unwrap();
    assert_eq!(
        loaded_a.status,
        ContractStatus::Ready,
        "snapshot update must roll back on mismatch"
    );
    assert!(store.list_events("c-a").unwrap().is_empty());
    assert!(store.list_events("c-b").unwrap().is_empty());
}
