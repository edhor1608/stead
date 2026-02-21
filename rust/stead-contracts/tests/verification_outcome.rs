use stead_contracts::{Contract, ContractStatus};

fn contract_in_verifying_state() -> Contract {
    let mut contract = Contract::new("c-verify", vec![]);
    contract.transition_to(ContractStatus::Claimed).unwrap();
    contract.transition_to(ContractStatus::Executing).unwrap();
    contract.transition_to(ContractStatus::Verifying).unwrap();
    contract
}

#[test]
fn verification_pass_moves_to_completed() {
    let mut contract = contract_in_verifying_state();

    let event = contract
        .finish_verification(true)
        .expect("verifying -> completed should be valid");

    assert_eq!(event.from, ContractStatus::Verifying);
    assert_eq!(event.to, ContractStatus::Completed);
    assert_eq!(contract.status, ContractStatus::Completed);
}

#[test]
fn verification_fail_moves_to_failed() {
    let mut contract = contract_in_verifying_state();

    let event = contract
        .finish_verification(false)
        .expect("verifying -> failed should be valid");

    assert_eq!(event.from, ContractStatus::Verifying);
    assert_eq!(event.to, ContractStatus::Failed);
    assert_eq!(contract.status, ContractStatus::Failed);
}
