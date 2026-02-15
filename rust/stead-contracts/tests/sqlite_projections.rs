use stead_contracts::{AttentionTier, Contract, ContractStatus, SqliteContractStore};

#[test]
fn anomalies_projection_returns_failed_and_rollback_states() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let mut failed = Contract::new("c-failed", vec![]);
    failed.status = ContractStatus::Failed;
    store.save_contract(&failed).unwrap();

    let mut rolling_back = Contract::new("c-rollback", vec![]);
    rolling_back.status = ContractStatus::RollingBack;
    store.save_contract(&rolling_back).unwrap();

    let mut rolled_back = Contract::new("c-rolled", vec![]);
    rolled_back.status = ContractStatus::RolledBack;
    store.save_contract(&rolled_back).unwrap();

    let mut completed = Contract::new("c-completed", vec![]);
    completed.status = ContractStatus::Completed;
    store.save_contract(&completed).unwrap();

    let anomalies = store.list_anomalies().unwrap();
    let ids: Vec<String> = anomalies.into_iter().map(|c| c.id).collect();

    assert!(ids.contains(&"c-failed".to_string()));
    assert!(ids.contains(&"c-rollback".to_string()));
    assert!(ids.contains(&"c-rolled".to_string()));
    assert!(!ids.contains(&"c-completed".to_string()));
}

#[test]
fn decisions_projection_returns_open_decisions_and_attention_mapping() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let ready = Contract::new("c-decision", vec![]);
    store.save_contract(&ready).unwrap();

    let mut executing = Contract::new("c-running", vec![]);
    executing.status = ContractStatus::Executing;
    store.save_contract(&executing).unwrap();

    store
        .create_decision("c-decision", "Choose API strategy")
        .unwrap();

    let decisions = store.list_open_decisions().unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].contract_id, "c-decision");

    let needs_decision = store
        .list_by_attention_tier(AttentionTier::NeedsDecision)
        .unwrap();
    assert_eq!(needs_decision.len(), 1);
    assert_eq!(needs_decision[0].id, "c-decision");

    let running = store
        .list_by_attention_tier(AttentionTier::Running)
        .unwrap();
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].id, "c-running");
}

#[test]
fn needs_decision_projection_deduplicates_contracts() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("contracts.db");
    let store = SqliteContractStore::open(&db_path).unwrap();

    let contract = Contract::new("c-dedupe", vec![]);
    store.save_contract(&contract).unwrap();

    store.create_decision("c-dedupe", "First").unwrap();
    store.create_decision("c-dedupe", "Second").unwrap();

    let needs_decision = store
        .list_by_attention_tier(AttentionTier::NeedsDecision)
        .unwrap();

    assert_eq!(needs_decision.len(), 1);
    assert_eq!(needs_decision[0].id, "c-dedupe");
}
