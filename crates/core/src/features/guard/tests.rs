use strum::IntoEnumIterator as _;

use super::{Guard, ReadState};

const READS: [ReadState; 2] = [ReadState::Failed, ReadState::Waiting];

#[test]
fn a_failed_locks_read_says_unread_and_a_due_one_says_waiting() {
    assert_eq!(
        Guard::UserLocks.hold_text(ReadState::Failed, ReadState::Waiting),
        "changes-unread-locks"
    );
    assert_eq!(
        Guard::UserLocks.hold_text(ReadState::Waiting, ReadState::Waiting),
        "changes-wait-locks"
    );
}

#[test]
fn the_aur_read_decides_the_aur_guard_alone() {
    assert_eq!(
        Guard::AurPackages.hold_text(ReadState::Waiting, ReadState::Failed),
        "changes-unread-aur"
    );
    assert_eq!(
        Guard::AurPackages.hold_text(ReadState::Waiting, ReadState::Waiting),
        "changes-wait-aur"
    );
}

#[test]
fn a_guard_without_a_read_of_its_own_waits_for_the_system() {
    let others =
        Guard::iter().filter(|guard| !matches!(guard, Guard::UserLocks | Guard::AurPackages));

    for guard in others {
        for locks in READS {
            for aur in READS {
                assert_eq!(
                    guard.hold_text(locks, aur),
                    "changes-wait-read",
                    "{guard:?}"
                );
            }
        }
    }
}
