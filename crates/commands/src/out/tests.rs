use toolbox_cli::Exit;
use toolbox_cli::output::{Paint, Style};

use super::failure_text;

const PLAIN: Paint = Paint::new(Style::Plain);

#[test]
fn a_failure_is_one_line_and_a_hint() {
    let text = failure_text(
        PLAIN,
        "updates could not be read",
        "==> ERROR: no fakeroot",
        false,
    );

    assert_eq!(
        text,
        "error: updates could not be read\nhint: add --verbose to see the program's own words\n"
    );
}

#[test]
fn verbose_shows_the_programs_words_under_the_line() {
    let text = failure_text(
        PLAIN,
        "updates could not be read",
        "==> ERROR: no fakeroot\n",
        true,
    );

    assert_eq!(
        text,
        "error: updates could not be read\n    ==> ERROR: no fakeroot\n"
    );
}

#[test]
fn a_failure_without_words_needs_no_hint() {
    assert_eq!(failure_text(PLAIN, "gone", " ", false), "error: gone\n");
}

#[test]
fn an_error_under_json_is_an_error_line() {
    assert_eq!(
        super::said(PLAIN, "no unit sshd", false),
        "error: no unit sshd\n"
    );
    assert_eq!(
        super::said(PLAIN, "no unit sshd", true),
        "{\"schema\":1,\"event\":\"error\",\"message\":\"no unit sshd\"}\n"
    );
}

#[test]
fn quiet_leaves_the_hint_out() {
    let quiet = PLAIN.quiet(true);

    assert_eq!(
        failure_text(quiet, "updates could not be read", "no fakeroot", false),
        "error: updates could not be read\n"
    );
    assert_eq!(quiet.hint("try again"), "");
}

#[test]
fn a_failure_under_json_carries_the_program_text() {
    assert_eq!(
        super::failed_text(PLAIN, "dkms failed", "no kernel\n", false, true),
        "{\"schema\":1,\"event\":\"error\",\"message\":\"dkms failed\",\"detail\":\"no kernel\\n\"}\n"
    );
}

#[test]
fn a_hint_of_an_error_gets_its_own_line() {
    assert_eq!(
        super::said(PLAIN, "no unit sshd\ndid you mean `sshd.service`?", false),
        "error: no unit sshd\nhint: did you mean `sshd.service`?\n"
    );
}

/// A read's `--json` keeps standard output for its JSON, so its error is
/// an object on standard error, with the exit code the command ends with.
#[test]
fn a_read_error_under_json_is_an_object_with_its_exit_code() {
    let said = super::Said::new("no unit sshd\ndid you mean `sshd.service`?", None);

    assert_eq!(
        super::error_object(&said, Exit::Unavailable),
        concat!(
            r#"{"schema":1,"error":{"message":"no unit sshd","#,
            r#""hint":"did you mean `sshd.service`?","code":5,"reason":"unavailable"}}"#,
            "\n"
        )
    );
}

#[test]
fn a_read_failure_under_json_carries_the_program_text() {
    let said = super::Said::new("updates could not be read", Some("no fakeroot\n"));

    assert_eq!(
        super::error_object(&said, Exit::Failure),
        concat!(
            r#"{"schema":1,"error":{"message":"updates could not be read","#,
            r#""detail":"no fakeroot\n","code":1,"reason":"failure"}}"#,
            "\n"
        )
    );
}

/// Under a read's `--json` standard error holds the JSON objects alone,
/// so a script parses every line of it. Text output keeps its `error:`
/// lines, and a change its JSON Lines on standard output.
#[test]
fn a_read_under_json_keeps_its_error_for_the_object_alone() {
    use super::{Json, Sink, sink};

    assert_eq!(sink(Json::Read), Sink::Kept);
    assert_eq!(sink(Json::Off), Sink::Text);
    assert_eq!(sink(Json::Lines), Sink::Line);
}

/// Standard output is gone, so under any `--json` the object on standard
/// error is all a script gets.
#[test]
fn lost_output_under_any_json_is_kept_for_its_object() {
    use super::{Json, Sink, lost_sink};

    assert_eq!(lost_sink(Json::Lines), Sink::Kept);
    assert_eq!(lost_sink(Json::Read), Sink::Kept);
    assert_eq!(lost_sink(Json::Off), Sink::Text);
}
