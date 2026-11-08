use toolbox_plan::{GrubState, InvalidName, NameKind, Reason};

use super::{Edit, EditProblem, edit, rule};
use crate::domains::CurrentGrub;
use crate::features::boot::{Flag, Refusal};
use crate::{ChangeSet, Snapshot};

fn grub() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentGrub(GrubState::read(
        "GRUB_DEFAULT=0\nGRUB_TIMEOUT=5\nGRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"\n",
    )));
    snapshot
}

fn code(change: Edit) -> &'static str {
    edit(change, &mut ChangeSet::default(), &grub()).map_or_else(|problem| problem.code(), |()| "")
}

#[test]
fn broken_values_name_their_rule() {
    assert_eq!(code(Edit::Default(String::new())), "empty");
    assert_eq!(code(Edit::Timeout(301)), "invalid");
    assert_eq!(
        code(Edit::AddParameter("a b".to_owned())),
        "invalid-character"
    );
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
    assert_eq!(
        edit(
            Edit::AddParameter("loglevel=3".to_owned()),
            &mut changes,
            &snapshot
        ),
        Ok(())
    );
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
    assert_eq!(problem.unwrap_err().code(), "unavailable");
}

#[test]
fn every_broken_rule_has_its_code() {
    let refused = |reason| rule(&InvalidName::new(NameKind::Package, "x", reason));

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
    assert_eq!(Edit::AddParameter("a b".to_owned()).value(), "a b");
    assert_eq!(Edit::Timeout(301).value(), "301");
    assert_eq!(Edit::Submenus(true).value(), "");
}
