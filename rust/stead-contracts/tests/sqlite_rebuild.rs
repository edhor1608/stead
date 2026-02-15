use stead_contracts::{Contract, ContractStatus, SqliteContractStore};

#[test]
fn rebuild_uses_event_log_as_source_of_truth() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let mut contract = Contract::new("c-rebuild", vec![]);
    store.save_contract(&contract).unwrap();

    let event_1 = contract.transition_to(ContractStatus::Claimed).unwrap();
    store.record_transition(&contract, &event_1).unwrap();

    let event_2 = contract.transition_to(ContractStatus::Executing).unwrap();
    store.record_transition(&contract, &event_2).unwrap();

    // Corrupt the snapshot intentionally to ensure rebuild prefers events.
    contract.status = ContractStatus::Pending;
    store.save_contract(&contract).unwrap();

    let rebuilt = store
        .rebuild_contract_from_events("c-rebuild")
        .unwrap()
        .expect("contract should rebuild");

    assert_eq!(rebuilt.status, ContractStatus::Executing);
}

#[test]
fn rebuild_without_events_falls_back_to_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let contract = Contract::new("c-no-events", vec!["dep-1".into()]);
    store.save_contract(&contract).unwrap();

    let rebuilt = store
        .rebuild_contract_from_events("c-no-events")
        .unwrap()
        .expect("snapshot exists");

    assert_eq!(rebuilt.status, ContractStatus::Pending);
    assert_eq!(rebuilt.blocked_by, vec!["dep-1"]);
}

#[test]
fn rebuild_from_events_restores_blocked_by_when_snapshot_corrupted() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let mut contract = Contract::new("c-events-only", vec!["dep-1".into(), "dep-2".into()]);
    store.save_contract(&contract).unwrap();

    let event = contract.transition_to(ContractStatus::Ready).unwrap();
    store.record_transition(&contract, &event).unwrap();

    contract.blocked_by.clear();
    store.save_contract(&contract).unwrap();

    let rebuilt = store
        .rebuild_contract_from_events("c-events-only")
        .unwrap()
        .expect("contract should rebuild from event history");

    assert_eq!(rebuilt.status, ContractStatus::Ready);
    assert_eq!(rebuilt.blocked_by, vec!["dep-1", "dep-2"]);
}
