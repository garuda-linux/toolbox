use std::collections::BTreeMap;

use toolbox_plan::{PlannedAction, RootAction, ShellPath};

use super::{CurrentShells, LoginShells};
use crate::domain::Summary;
use crate::test_support::{actions, user_name};
use crate::{ChangeSet, Snapshot, SummaryCode};

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentShells(BTreeMap::from([(
        user_name("nico"),
        shell("/usr/bin/fish"),
    )])));
    snapshot
}

#[test]
fn the_current_shell_drops_out() {
    let mut changes = ChangeSet::default();
    changes.want::<LoginShells>(user_name("nico"), shell("/usr/bin/fish"), &snapshot());

    assert!(changes.is_empty());
}

#[test]
fn a_new_shell_runs_set_shell_for_its_user() {
    let mut changes = ChangeSet::default();
    changes.want::<LoginShells>(user_name("nico"), shell("/usr/bin/zsh"), &snapshot());

    let steps = actions(&changes);

    assert_eq!(
        steps,
        [PlannedAction::Root(RootAction::SetShell {
            user: user_name("nico"),
            shell: shell("/usr/bin/zsh"),
        })]
    );
}

#[test]
fn an_account_without_a_known_shell_keeps_the_wish() {
    let mut changes = ChangeSet::default();
    changes.want::<LoginShells>(user_name("other"), shell("/usr/bin/fish"), &snapshot());

    assert_eq!(changes.len(), 1);
}

#[test]
fn the_review_names_the_user_and_the_new_shell() {
    let mut changes = ChangeSet::default();
    changes.want::<LoginShells>(user_name("nico"), shell("/usr/bin/zsh"), &snapshot());

    let views = changes.views();
    let view = views.first().unwrap();

    assert_eq!(view.domain, "login-shells");
    assert_eq!(view.key, "nico");
    assert_eq!(
        view.summary,
        Summary::pair(SummaryCode::LoginShell, "nico", "/usr/bin/zsh")
    );
}
