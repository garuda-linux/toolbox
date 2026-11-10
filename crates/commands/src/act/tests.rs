use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_cli::output::Mark;

use super::{TaskRow, task_listing};
use crate::change::Refused;
use crate::out::{Out, Settings};

fn row(id: &str, available: bool, asks_first: bool) -> TaskRow {
    TaskRow {
        id: id.to_owned(),
        available,
        asks_first,
    }
}

#[test]
fn tasks_are_marked_by_whether_they_can_run_here() {
    let rows = [
        row("refresh-keyring", true, false),
        row("clean-package-cache", true, true),
        row("grub-fix", false, false),
    ];

    let listing = task_listing(&rows);

    assert_eq!(listing.title, "Maintenance tasks");
    assert_eq!(listing.summary.as_deref(), Some("3, 2 available"));
    let no_head: &[&str] = &[];
    assert_eq!(listing.head, no_head);
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox maintenance run <task>")
    );

    let marks: Vec<Mark> = listing.rows.iter().map(|row| row.mark).collect();
    assert_eq!(marks, [Mark::On, Mark::On, Mark::Off]);
    assert_eq!(listing.rows[1].cells, ["clean-package-cache", "asks first"]);
    assert_eq!(listing.rows[2].cells, ["grub-fix", "not available"]);
}

#[test]
fn a_refusal_under_json_is_one_error_line() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let refused = Refused::Unknown("no unit sshd\ndid you mean `sshd.service`?".to_owned());
    let confirm = Confirm {
        yes: true,
        json: true,
        ..Confirm::default()
    };

    let mut out = Vec::new();
    let exit = super::run(
        Err(refused),
        confirm,
        None,
        &runtime,
        &mut Out::new(&mut out, Settings::default()),
    );

    assert_eq!(exit, Exit::Unavailable);
    assert_eq!(
        String::from_utf8(out).unwrap(),
        concat!(
            r#"{"schema":1,"event":"error","message":"no unit sshd","hint":"did you mean `sshd.service`?"}"#,
            "\n"
        )
    );
}

/// A change that would do nothing is an empty plan that ended well, as a
/// switch already as asked is, never an error.
#[test]
fn nothing_to_do_under_json_is_an_empty_plan_and_ends_with_0() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let nothing = Refused::NothingToDo("Nothing to do: sshd.service runs already.".to_owned());
    let confirm = Confirm {
        yes: true,
        json: true,
        ..Confirm::default()
    };

    let mut out = Vec::new();
    let exit = super::run(
        Err(nothing),
        confirm,
        None,
        &runtime,
        &mut Out::new(&mut out, Settings::default()),
    );

    assert_eq!(exit, Exit::Success);
    assert_eq!(
        String::from_utf8(out).unwrap(),
        concat!(
            r#"{"schema":1,"event":"plan","steps":[]}"#,
            "\n",
            r#"{"schema":1,"event":"done","ok":true}"#,
            "\n"
        )
    );
}
