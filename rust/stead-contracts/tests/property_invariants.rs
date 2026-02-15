use proptest::prelude::*;
use stead_contracts::{Contract, ContractStatus};

fn status_strategy() -> impl Strategy<Value = ContractStatus> {
    prop_oneof![
        Just(ContractStatus::Pending),
        Just(ContractStatus::Ready),
        Just(ContractStatus::Claimed),
        Just(ContractStatus::Executing),
        Just(ContractStatus::Verifying),
        Just(ContractStatus::Completed),
        Just(ContractStatus::Failed),
        Just(ContractStatus::RollingBack),
        Just(ContractStatus::RolledBack),
        Just(ContractStatus::Cancelled),
    ]
}

proptest! {
    #[test]
    fn successful_transition_updates_status_and_event(target in status_strategy()) {
        let mut contract = Contract::new("prop-1", vec![]);

        if let Ok(event) = contract.transition_to(target) {
            prop_assert_eq!(event.from, ContractStatus::Ready);
            prop_assert_eq!(event.to, target);
            prop_assert_eq!(contract.status, target);
        } else {
            prop_assert_eq!(contract.status, ContractStatus::Ready);
        }
    }

    #[test]
    fn transition_relations_are_consistent(from in status_strategy(), to in status_strategy()) {
        let relation = from.can_transition_to(to);
        let listed = from.valid_transitions().contains(&to);
        prop_assert_eq!(relation, listed);
    }
}
