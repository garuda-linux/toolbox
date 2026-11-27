use super::packages;
use crate::action::RootAction;
use crate::action::testing::shown;
use crate::procedure::{Command, Invocation, Program};

fn removal(names: &[&str]) -> RootAction {
    RootAction::RemoveMakeDependencies {
        packages: packages(names),
    }
}

#[test]
fn the_query_lists_the_orphans_and_the_removal_takes_only_those_without_s() {
    assert_eq!(
        removal(&["cowsay"]).procedure(),
        [Invocation::RunWithOrphansFrom {
            query: Command::new(Program::Pacman).args(["-Qdtq", "--", "cowsay"]),
            run: Command::new(Program::Pacman).args(["-Rn", "--noconfirm", "--"]),
        }]
    );
}

#[test]
fn the_review_says_only_orphans_go_and_never_with_their_dependencies() {
    assert_eq!(
        shown(&removal(&["cowsay", "tree"])),
        [
            "pacman -Rn --noconfirm -- $(pacman -Qdtq -- cowsay tree)  # only the names pacman -Qdtq lists, installed as dependencies and required by nothing, never with -s"
        ]
    );
}
