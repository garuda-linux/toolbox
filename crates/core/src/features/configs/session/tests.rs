mod home;

use toolbox_plan::{ConfigFilePath, ConfigText, ConfigWrite, EditCopy, Editor};

use super::{
    Changes, Compared, EditWrite, EditedFile, Fetched, Phase, Problem, Session, Stage, Work,
};

const OLD: &str = "a = 1\n";
const NEW: &str = "a = 2\n";

struct Shown {
    phase: &'static str,
    path: String,
    target: String,
    editor: &'static str,
    backup: String,
    added: usize,
    removed: usize,
}

fn path() -> ConfigFilePath {
    ConfigFilePath::new("/etc/fstab").unwrap()
}

fn copy() -> EditCopy {
    EditCopy::new("/run/user/1000/garuda-toolbox/edit/fstab").unwrap()
}

fn text(content: &str) -> ConfigText {
    ConfigText::new(content).unwrap()
}

fn fetched() -> Fetched {
    Fetched {
        editor: Editor::Micro,
        copy: copy(),
        original: text(OLD),
        target: path().into(),
    }
}

fn changed(original: &str, edited: &str) -> Compared {
    Compared::Changed(Changes {
        original: text(original),
        edited: text(edited),
        plain: "-a = 1\n+a = 2\n".to_owned(),
        colored: String::new(),
        added: 1,
        removed: 1,
    })
}

fn editing() -> Session {
    let mut session = Session::default();

    assert_eq!(session.start(path()), Ok(Work::Fetch(path().into())));
    assert_eq!(
        session.fetched(Ok(fetched())),
        Some(Work::RunEditor(Editor::Micro, copy()))
    );
    assert_eq!(session.plan_started(), None);
    assert_eq!(session.phase(), Phase::Running(Stage::Editor));
    session
}

fn reviewing() -> Session {
    let mut session = editing();

    assert_eq!(
        session.plan_finished(true),
        Some(Work::Compare(copy(), text(OLD)))
    );
    assert_eq!(session.compared(Ok(changed(OLD, NEW))), None);
    assert_eq!(session.phase(), Phase::Review);
    session
}

fn code(session: &Session) -> String {
    session.problem().code.to_owned()
}

fn shown(session: &Session) -> Shown {
    let text = |path: Option<&EditedFile>| path.map(ToString::to_string).unwrap_or_default();

    Shown {
        phase: session.phase().code(),
        path: text(session.path()),
        target: text(session.target()),
        editor: session
            .editor()
            .map_or("", |editor| editor.program().name()),
        backup: session.backup().unwrap_or_default(),
        added: session.changes().map_or(0, |changes| changes.added),
        removed: session.changes().map_or(0, |changes| changes.removed),
    }
}

#[test]
fn an_edit_goes_from_reading_to_saved() {
    let mut session = reviewing();
    let view = shown(&session);

    assert_eq!(view.phase, "review");
    assert_eq!(view.path, "/etc/fstab");
    assert_eq!(view.target, "/etc/fstab");
    assert_eq!(view.editor, "micro");
    assert_eq!(view.backup, "/var/lib/garuda-toolbox/backups/etc/fstab");
    assert_eq!((view.added, view.removed), (1, 1));

    assert_eq!(
        session.apply(),
        Some(Work::Write(EditWrite::Root(ConfigWrite {
            path: path(),
            content: text(NEW),
            original: text(OLD).fingerprint(),
        })))
    );
    assert_eq!(shown(&session).phase, "writing");
    assert_eq!(session.plan_started(), None);
    assert_eq!(session.plan_finished(true), Some(Work::RemoveCopy(copy())));
    assert_eq!(session.phase(), Phase::Saved);
    assert_eq!(session.changes(), None);

    session.dismiss();
    assert_eq!(session, Session::default());
}

#[test]
fn one_edit_at_a_time() {
    let mut reading = Session::default();

    reading.start(path()).unwrap();
    assert_eq!(reading.start(path()), Err("busy"));
    assert_eq!(editing().start(path()), Err("busy"));
    assert_eq!(reviewing().start(path()), Err("busy"));

    let mut saved = reviewing();

    saved.apply();
    saved.plan_started();
    saved.plan_finished(true);
    saved.start(path()).unwrap();
    assert_eq!(shown(&saved).phase, "reading");
}

#[test]
fn a_failed_read_ends_the_edit_with_its_reason() {
    let mut session = Session::default();

    session.start(path()).unwrap();
    assert_eq!(session.fetched(Err(Problem::new("dismissed", ""))), None);
    assert_eq!(session.phase(), Phase::Failed);
    assert_eq!(code(&session), "dismissed");

    session.dismiss();
    assert_eq!(session.phase(), Phase::Idle);
}

#[test]
fn an_unchanged_copy_is_removed_and_said() {
    let mut session = editing();

    session.plan_finished(true);
    assert_eq!(
        session.compared(Ok(Compared::Unchanged)),
        Some(Work::RemoveCopy(copy()))
    );
    assert_eq!(session.phase(), Phase::Unchanged);
}

#[test]
fn an_editor_that_failed_without_a_change_is_a_failure() {
    let mut session = editing();

    session.plan_finished(false);
    assert_eq!(
        session.compared(Ok(Compared::Unchanged)),
        Some(Work::RemoveCopy(copy()))
    );
    assert_eq!(session.phase(), Phase::Failed);
    assert_eq!(code(&session), "editor-failed");

    let mut saved_then_crashed = editing();

    saved_then_crashed.plan_finished(false);
    assert_eq!(saved_then_crashed.compared(Ok(changed(OLD, NEW))), None);
    assert_eq!(saved_then_crashed.phase(), Phase::Review);
}

#[test]
fn a_copy_that_is_no_text_ends_the_edit() {
    let mut session = editing();

    session.plan_finished(true);
    assert_eq!(
        session.compared(Err(Problem::new("not-text", "the file is no text"))),
        Some(Work::RemoveCopy(copy()))
    );
    assert_eq!(session.phase(), Phase::Failed);
    assert_eq!(code(&session), "not-text");
}

/// The other plan's end must never count as this edit's: that would
/// remove the copy of an edit that was not written.
#[test]
fn a_refused_plan_is_not_mistaken_for_a_finished_one() {
    let mut session = reviewing();

    session.apply().unwrap();
    assert_eq!(session.phase(), Phase::Starting(Stage::Write));
    assert_eq!(session.plan_finished(true), None, "another plan ended");
    assert_eq!(session.phase(), Phase::Starting(Stage::Write));
    assert_eq!(session.plan_refused(), None);
    assert_eq!(session.phase(), Phase::Review);
    assert_eq!(code(&session), "busy");
    assert_eq!(session.plan_finished(true), None, "the other plan ended");
    assert_eq!(session.phase(), Phase::Review);
    assert!(session.changes().is_some());

    let mut first = Session::default();

    first.start(path()).unwrap();
    first.fetched(Ok(fetched()));
    assert_eq!(first.plan_refused(), Some(Work::RemoveCopy(copy())));
    assert_eq!(first.phase(), Phase::Failed);
    assert_eq!(code(&first), "busy");
}

#[test]
fn a_failed_write_keeps_the_edit_for_another_try() {
    let mut session = reviewing();

    session.apply();
    session.plan_started();
    assert_eq!(session.plan_finished(false), None);
    assert_eq!(session.phase(), Phase::Review);
    assert_eq!(code(&session), "write-failed");
    assert_eq!(session.changes().unwrap().edited, text(NEW));

    assert_eq!(
        session.reread(),
        Some(Work::Reread(path().into(), text(NEW)))
    );
    assert_eq!(shown(&session).phase, "comparing");
    assert_eq!(session.compared(Ok(changed("a = 9\n", NEW))), None);
    assert_eq!(code(&session), "");
    assert_eq!(
        session.apply(),
        Some(Work::Write(EditWrite::Root(ConfigWrite {
            path: path(),
            content: text(NEW),
            original: text("a = 9\n").fingerprint(),
        })))
    );
}

#[test]
fn a_file_that_cannot_be_read_again_keeps_the_edit() {
    let mut session = reviewing();

    session.reread().unwrap();
    assert_eq!(session.compared(Err(Problem::new("dismissed", ""))), None);
    assert_eq!(session.phase(), Phase::Review);
    assert_eq!(code(&session), "dismissed");
    assert!(session.changes().is_some());
}

#[test]
fn editing_again_opens_the_same_copy() {
    let mut session = reviewing();

    assert_eq!(
        session.edit_again(),
        Some(Work::RunEditor(Editor::Micro, copy()))
    );
    assert_eq!(shown(&session).phase, "editing");
    session.plan_started();
    assert_eq!(
        session.plan_finished(true),
        Some(Work::Compare(copy(), text(OLD)))
    );

    assert_eq!(
        session.compared(Ok(Compared::Unchanged)),
        Some(Work::RemoveCopy(copy()))
    );
    assert_eq!(session.phase(), Phase::Unchanged);
    assert_eq!(session.changes(), None);
}

#[test]
fn discarding_a_review_removes_the_copy() {
    let mut session = reviewing();

    assert_eq!(session.discard(), Some(Work::RemoveCopy(copy())));
    assert_eq!(session, Session::default());
}

#[test]
fn giving_up_while_the_editor_runs_stops_it_then_removes_the_copy() {
    let mut session = editing();

    assert_eq!(session.discard(), Some(Work::Cancel));
    assert_eq!(session.phase(), Phase::Running(Stage::Editor));
    assert_eq!(session.plan_finished(false), Some(Work::RemoveCopy(copy())));
    assert_eq!(session, Session::default());
}

/// Before the plan started a cancel could hit another page's plan, so it
/// waits until this edit's own plan runs.
#[test]
fn giving_up_before_the_editor_started_cancels_only_its_own_plan() {
    let mut session = Session::default();

    session.start(path()).unwrap();
    session.fetched(Ok(fetched()));
    assert_eq!(session.discard(), None);
    assert_eq!(session.plan_started(), Some(Work::Cancel));
    assert_eq!(session.plan_finished(false), Some(Work::RemoveCopy(copy())));
    assert_eq!(session, Session::default());

    let mut refused = Session::default();

    refused.start(path()).unwrap();
    refused.fetched(Ok(fetched()));
    refused.discard();
    assert_eq!(refused.plan_refused(), Some(Work::RemoveCopy(copy())));
    assert_eq!(refused, Session::default());
}

#[test]
fn nothing_is_discarded_while_a_write_runs_and_late_notes_do_nothing() {
    let mut session = reviewing();

    session.apply();
    session.plan_started();
    assert_eq!(session.discard(), None);
    assert_eq!(session.phase(), Phase::Running(Stage::Write));

    let mut idle = Session::default();

    assert_eq!(idle.plan_started(), None);
    assert_eq!(idle.plan_refused(), None);
    assert_eq!(idle.plan_finished(true), None);
    assert_eq!(idle.compared(Ok(Compared::Unchanged)), None);
    assert_eq!(idle.apply(), None);
    assert_eq!(idle.edit_again(), None);
    assert_eq!(idle.reread(), None);
    assert_eq!(idle.discard(), None);
    assert_eq!(
        idle.fetched(Ok(fetched())),
        Some(Work::RemoveCopy(copy())),
        "a copy nobody waits for must not stay"
    );
    assert_eq!(idle, Session::default());
}

/// The edit of a link is the edit of the file it leads to: what the read
/// named is what the dialog shows, what is written and what is read again.
#[test]
fn the_edit_of_a_link_goes_on_with_the_file_it_leads_to() {
    let real = ConfigFilePath::new("/etc/real/fstab").unwrap();
    let mut session = Session::default();

    session.start(path()).unwrap();
    assert_eq!(shown(&session).target, "/etc/fstab");
    session.fetched(Ok(Fetched {
        target: real.clone().into(),
        ..fetched()
    }));
    session.plan_started();
    session.plan_finished(true);
    session.compared(Ok(changed(OLD, NEW)));

    let view = shown(&session);

    assert_eq!(view.path, "/etc/fstab");
    assert_eq!(view.target, "/etc/real/fstab");
    assert_eq!(
        view.backup,
        "/var/lib/garuda-toolbox/backups/etc/real/fstab"
    );
    assert_eq!(
        session.clone().reread(),
        Some(Work::Reread(real.clone().into(), text(NEW)))
    );
    assert_eq!(
        session.apply(),
        Some(Work::Write(EditWrite::Root(ConfigWrite {
            path: real,
            content: text(NEW),
            original: text(OLD).fingerprint(),
        })))
    );
}
