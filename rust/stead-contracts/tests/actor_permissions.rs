use stead_contracts::{Actor, TransitionAction};

#[test]
fn permission_matrix_matches_spec() {
    let cases = [
        (TransitionAction::DepsMet, Actor::System, true),
        (TransitionAction::DepsMet, Actor::Agent, false),
        (TransitionAction::DepsMet, Actor::Human, false),
        (TransitionAction::Claim, Actor::System, false),
        (TransitionAction::Claim, Actor::Agent, true),
        (TransitionAction::Claim, Actor::Human, true),
        (TransitionAction::Unclaim, Actor::System, false),
        (TransitionAction::Unclaim, Actor::Agent, true),
        (TransitionAction::Unclaim, Actor::Human, true),
        (TransitionAction::Start, Actor::System, false),
        (TransitionAction::Start, Actor::Agent, true),
        (TransitionAction::Start, Actor::Human, true),
        (TransitionAction::Verify, Actor::System, false),
        (TransitionAction::Verify, Actor::Agent, true),
        (TransitionAction::Verify, Actor::Human, true),
        (TransitionAction::Pass, Actor::System, true),
        (TransitionAction::Pass, Actor::Agent, false),
        (TransitionAction::Pass, Actor::Human, false),
        (TransitionAction::Fail, Actor::System, true),
        (TransitionAction::Fail, Actor::Agent, false),
        (TransitionAction::Fail, Actor::Human, false),
        (TransitionAction::Rollback, Actor::System, false),
        (TransitionAction::Rollback, Actor::Agent, true),
        (TransitionAction::Rollback, Actor::Human, true),
        (TransitionAction::RollbackDone, Actor::System, true),
        (TransitionAction::RollbackDone, Actor::Agent, false),
        (TransitionAction::RollbackDone, Actor::Human, false),
        (TransitionAction::Cancel, Actor::System, false),
        (TransitionAction::Cancel, Actor::Agent, false),
        (TransitionAction::Cancel, Actor::Human, true),
    ];

    for (action, actor, expected) in cases {
        assert_eq!(
            action.is_allowed_for(actor),
            expected,
            "permission mismatch for {action:?} and {actor:?}",
        );
    }
}
