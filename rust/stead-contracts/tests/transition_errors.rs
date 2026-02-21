use stead_contracts::{ContractStatus, TransitionError};

#[test]
fn invalid_transition_returns_typed_error_with_from_and_to() {
    let error = ContractStatus::Pending
        .transition_to(ContractStatus::Completed)
        .expect_err("pending -> completed must be rejected");

    assert_eq!(
        error,
        TransitionError {
            from: ContractStatus::Pending,
            to: ContractStatus::Completed,
        }
    );
}

#[test]
fn terminal_states_are_immutable_via_transition_api() {
    for terminal in [
        ContractStatus::Completed,
        ContractStatus::RolledBack,
        ContractStatus::Cancelled,
    ] {
        let error = terminal
            .transition_to(ContractStatus::Pending)
            .expect_err("terminal state should be immutable");

        assert_eq!(error.from, terminal);
        assert_eq!(error.to, ContractStatus::Pending);
    }
}
