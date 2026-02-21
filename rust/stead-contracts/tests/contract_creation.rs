use stead_contracts::{Contract, ContractStatus};

#[test]
fn contract_without_dependencies_starts_ready() {
    let contract = Contract::new("c-1", vec![]);

    assert_eq!(contract.status, ContractStatus::Ready);
    assert!(contract.blocked_by.is_empty());
}

#[test]
fn contract_with_dependencies_starts_pending() {
    let contract = Contract::new("c-2", vec!["dep-1".into(), "dep-2".into()]);

    assert_eq!(contract.status, ContractStatus::Pending);
    assert_eq!(contract.blocked_by, vec!["dep-1", "dep-2"]);
}
