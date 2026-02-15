use stead_contracts::{Contract, SqliteContractStore};

#[test]
fn list_contracts_returns_all_sorted_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    store.save_contract(&Contract::new("c-b", vec![])).unwrap();
    store.save_contract(&Contract::new("c-a", vec![])).unwrap();

    let contracts = store.list_contracts().unwrap();
    let ids: Vec<String> = contracts.into_iter().map(|c| c.id).collect();

    assert_eq!(ids, vec!["c-a", "c-b"]);
}
