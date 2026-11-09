use super::every_repair;
use crate::procedure::{Invocation, Program};
use crate::protocol::Leftover;

/// A failed cleanup names what it would have undone, so the Console and
/// the next repair's refusal say where.
#[test]
fn every_unmount_names_its_own_place_as_the_leftover() {
    for action in every_repair() {
        for invocation in action.procedure() {
            if let Invocation::Cleanup {
                command,
                leftover: Leftover::Mounted { place },
            } = &invocation
            {
                assert_eq!(command.program(), Program::Umount, "{action:?}");
                assert_eq!(
                    command.arguments().last(),
                    Some(&place.to_string()),
                    "{action:?}"
                );
            }
        }
    }
}

#[test]
fn an_unlock_is_closed_by_the_last_always_step_and_only_then() {
    for action in every_repair() {
        let procedure = action.procedure();
        let opens = procedure
            .iter()
            .filter(|invocation| {
                matches!(invocation, Invocation::Feed { command, input: _ }
                    if command.program() == Program::Cryptsetup)
            })
            .count();

        let closes_last = matches!(procedure.last(), Some(Invocation::Cleanup { command, leftover: Leftover::Open })
            if command.program() == Program::Cryptsetup
                && command.arguments() == ["close", "garuda-toolbox-repair"]);

        assert_eq!(opens, usize::from(action.needs_secret()), "{action:?}");
        assert_eq!(closes_last, action.needs_secret(), "{action:?}");
    }
}
