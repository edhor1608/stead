use stead_contracts::{Contract, ContractStatus};

#[test]
fn rollback_allowed_only_from_failed_state() {
    let mut contract = Contract::new("c-rb", vec![]);

    contract.transition_to(ContractStatus::Claimed).unwrap();
    contract.transition_to(ContractStatus::Executing).unwrap();
    contract.transition_to(ContractStatus::Failed).unwrap();

    let event = contract
        .rollback()
        .expect("failed -> rolling back is valid");
    assert_eq!(event.from, ContractStatus::Failed);
    assert_eq!(event.to, ContractStatus::RollingBack);
}

#[test]
fn rollback_rejected_when_not_failed() {
    let mut contract = Contract::new("c-rb-invalid", vec![]);

    let result = contract.rollback();
    assert!(result.is_err(), "ready -> rolling back must be rejected");
    assert_eq!(contract.status, ContractStatus::Ready);
}

#[test]
fn cancel_rejected_while_verifying() {
    let mut contract = Contract::new("c-cancel", vec![]);

    contract.transition_to(ContractStatus::Claimed).unwrap();
    contract.transition_to(ContractStatus::Executing).unwrap();
    contract.transition_to(ContractStatus::Verifying).unwrap();

    assert!(contract.cancel().is_err());
    assert_eq!(contract.status, ContractStatus::Verifying);
}

#[test]
fn cancel_allowed_from_executing() {
    let mut contract = Contract::new("c-cancel-ok", vec![]);

    contract.transition_to(ContractStatus::Claimed).unwrap();
    contract.transition_to(ContractStatus::Executing).unwrap();

    let event = contract.cancel().expect("executing -> cancelled is valid");
    assert_eq!(event.to, ContractStatus::Cancelled);
    assert_eq!(contract.status, ContractStatus::Cancelled);
}
