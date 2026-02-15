use stead_contracts::{Contract, ContractEvent, ContractStatus};

#[test]
fn emits_event_for_valid_transition() {
    let mut contract = Contract::new("c-1", vec![]);

    let event = contract
        .transition_to(ContractStatus::Claimed)
        .expect("ready -> claimed should be valid");

    assert_eq!(
        event,
        ContractEvent {
            contract_id: "c-1".to_string(),
            from: ContractStatus::Ready,
            to: ContractStatus::Claimed,
        }
    );
    assert_eq!(contract.status, ContractStatus::Claimed);
}

#[test]
fn no_event_emitted_for_invalid_transition() {
    let mut contract = Contract::new("c-2", vec![]);

    let result = contract.transition_to(ContractStatus::Completed);
    assert!(result.is_err());
    assert_eq!(contract.status, ContractStatus::Ready);
}
