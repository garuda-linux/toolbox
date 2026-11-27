use toolbox_plan::{GrubState, GrubTimeout, InvalidName, NameKind, Reason};

use super::{Edit, EditProblem, edit, rule};
use crate::domains::CurrentGrub;
use crate::features::RefusalText as _;
use crate::features::boot::view::Page;
use crate::features::boot::{Flag, Refusal};
use crate::{ChangeSet, Snapshot};

fn grub() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentGrub(GrubState::read(
        toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB,
    )));
    snapshot
}

fn code(change: Edit) -> &'static str {
    edit(change, &mut ChangeSet::default(), &grub())
        .map_or_else(|problem| problem.code().code(), |()| "")
}

fn add(text: &str) -> Edit {
    Edit::AddParameter {
        text: text.to_owned(),
        confirmed: false,
    }
}

#[test]
fn broken_values_name_their_rule() {
    assert_eq!(code(Edit::Default(String::new())), "empty");
    assert_eq!(code(Edit::Timeout(301)), "invalid");
    assert_eq!(code(add("a b")), "invalid-character");
    assert_eq!(
        code(Edit::Background("/etc/passwd".to_owned())),
        "wrong-type"
    );
    assert_eq!(code(Edit::Theme("/etc/passwd".to_owned())), "wrong-type");
    assert_eq!(
        code(Edit::PlymouthTheme("no way".to_owned())),
        "invalid-character"
    );
}

#[test]
fn good_values_are_queued() {
    let mut changes = ChangeSet::default();
    let snapshot = grub();

    assert_eq!(edit(Edit::Timeout(10), &mut changes, &snapshot), Ok(()));
    assert_eq!(edit(add("loglevel=3"), &mut changes, &snapshot), Ok(()));
    assert_eq!(
        edit(Edit::Flag(Flag::SysRq, true), &mut changes, &snapshot),
        Ok(())
    );
    assert!(!changes.is_empty());
}

#[test]
fn an_empty_path_takes_the_picture_away() {
    assert_eq!(code(Edit::Background(String::new())), "");
    assert_eq!(code(Edit::Theme(String::new())), "");
}

#[test]
fn without_grub_every_edit_is_refused() {
    let problem = edit(
        Edit::Submenus(false),
        &mut ChangeSet::default(),
        &Snapshot::default(),
    );

    assert_eq!(problem, Err(EditProblem::Refused(Refusal::Unavailable)));
    assert_eq!(
        problem.unwrap_err().code().text_id(),
        Some("boot-refused-unavailable")
    );
}

#[test]
fn every_broken_rule_has_its_code() {
    let refused = |reason| rule(&InvalidName::new(NameKind::Package, "x", reason)).code();

    assert_eq!(refused(Reason::Empty), "empty");
    assert_eq!(refused(Reason::TooLong { max: 8 }), "too-long");
    assert_eq!(refused(Reason::InvalidStart('-')), "invalid-character");
    assert_eq!(refused(Reason::InvalidCharacter(' ')), "invalid-character");
    assert_eq!(refused(Reason::InvalidComponent), "invalid-path");
    assert_eq!(refused(Reason::WrongFileType), "wrong-type");
    assert_eq!(refused(Reason::UnknownUnitType), "invalid");
    assert_eq!(refused(Reason::UnknownDevice), "invalid");
}

#[test]
fn an_edit_names_the_value_a_refusal_shows() {
    assert_eq!(add("a b").value(), "a b");
    assert_eq!(Edit::Timeout(301).value(), "301");
    assert_eq!(Edit::Submenus(true).value(), "");
}

/// `init=/bin/sh` boots into a root shell, so the page asks first.
#[test]
fn a_risky_parameter_waits_for_a_confirm() {
    let mut changes = ChangeSet::default();
    let snapshot = grub();
    let asked = |confirmed| Edit::AddParameter {
        text: "init=/bin/sh".to_owned(),
        confirmed,
    };

    assert_eq!(code(asked(false)), "needs-confirm");
    assert_eq!(
        edit(asked(false), &mut changes, &snapshot),
        Err(EditProblem::NeedsConfirm)
    );
    assert!(changes.is_empty());
    assert_eq!(edit(asked(true), &mut changes, &snapshot), Ok(()));
    assert!(!changes.is_empty());
}

#[test]
fn only_a_risky_parameter_asks_instead_of_refusing() {
    assert!(EditProblem::NeedsConfirm.code().needs_confirm());
    assert!(!EditProblem::OutOfRange.code().needs_confirm());
}

/// GRUB's -1 is the menu that waits for a key. No other negative number
/// is written.
#[test]
fn a_timeout_of_minus_one_queues_waiting_for_a_key() {
    let mut changes = ChangeSet::default();
    let snapshot = grub();

    assert_eq!(edit(Edit::Timeout(-1), &mut changes, &snapshot), Ok(()));
    assert_eq!(
        Page::read(&snapshot, &changes).timeout(),
        GrubTimeout::WaitForKey
    );
    assert_eq!(code(Edit::Timeout(-2)), "invalid");
}
