use std::collections::BTreeSet;

use stead_contracts::ContractStatus;

fn all_statuses() -> Vec<ContractStatus> {
    vec![
        ContractStatus::Pending,
        ContractStatus::Ready,
        ContractStatus::Claimed,
        ContractStatus::Executing,
        ContractStatus::Verifying,
        ContractStatus::Completed,
        ContractStatus::Failed,
        ContractStatus::RollingBack,
        ContractStatus::RolledBack,
        ContractStatus::Cancelled,
    ]
}

fn allowed_edges() -> BTreeSet<(ContractStatus, ContractStatus)> {
    use ContractStatus::*;

    [
        (Pending, Ready),
        (Pending, Cancelled),
        (Ready, Claimed),
        (Ready, Cancelled),
        (Claimed, Executing),
        (Claimed, Ready),
        (Claimed, Cancelled),
        (Executing, Verifying),
        (Executing, Failed),
        (Executing, Cancelled),
        (Verifying, Completed),
        (Verifying, Failed),
        (Failed, Ready),
        (Failed, RollingBack),
        (Failed, Cancelled),
        (RollingBack, RolledBack),
        (RollingBack, Failed),
    ]
    .into_iter()
    .collect()
}

#[test]
fn transition_matrix_matches_spec() {
    let allowed = allowed_edges();

    for from in all_statuses() {
        for to in all_statuses() {
            let expected = allowed.contains(&(from, to));
            assert_eq!(
                from.can_transition_to(to),
                expected,
                "unexpected transition result: {from:?} -> {to:?}",
            );
        }
    }
}

#[test]
fn terminal_states_have_no_outgoing_transitions() {
    use ContractStatus::*;

    for terminal in [Completed, RolledBack, Cancelled] {
        assert!(terminal.valid_transitions().is_empty());
    }
}
