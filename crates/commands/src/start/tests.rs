use toolbox_cli::Exit;

use super::{clap_said, clap_text, json_asked, usage_text};
use crate::out::error_object;

#[test]
fn json_is_found_in_the_raw_arguments_before_a_double_dash() {
    assert!(json_asked(["status", "--json", "--nope"]));
    assert!(json_asked(["--nope", "users", "list", "--json"]));
    assert!(!json_asked(["status", "--nope"]));
    assert!(!json_asked(["packages", "search", "--", "--json"]));
}

/// A wrong argument under `--json` is the same object a read's error is,
/// so a script parses standard error alike.
#[test]
fn a_wrong_argument_under_json_is_an_error_object_with_code_2() {
    let error = toolbox_cli::command()
        .try_get_matches_from(["garuda-toolbox", "status", "--json", "--nope"])
        .unwrap_err();

    let shown = error_object(&clap_said(&error.render().to_string()), Exit::Usage);
    let parsed: serde_json::Value = serde_json::from_str(&shown).unwrap();

    assert_eq!(parsed["schema"], 1_u64);
    assert_eq!(
        parsed["error"]["message"],
        "unexpected argument '--nope' found"
    );
    assert_eq!(parsed["error"]["code"], 2_u64);
    assert_eq!(parsed["error"]["reason"], "usage");
    assert!(
        shown.ends_with("}\n") && shown.matches('\n').count() == 1,
        "{shown}"
    );
}

#[test]
fn a_tip_of_clap_becomes_the_hint() {
    let said = clap_said(
        "error: unexpected argument '--foo' found\n\n  tip: to pass '--foo' as a value, use '-- --foo'\n\nUsage: garuda-toolbox status [OPTIONS]\n",
    );

    assert_eq!(
        said,
        crate::out::Said::new(
            "unexpected argument '--foo' found\nto pass '--foo' as a value, use '-- --foo'",
            None
        )
    );
}

#[test]
fn a_start_option_under_json_is_an_object_else_text() {
    assert_eq!(
        usage_text("`nowhere` is no page", true),
        concat!(
            r#"{"schema":1,"error":{"message":"`nowhere` is no page","#,
            r#""code":2,"reason":"usage"}}"#,
            "\n"
        )
    );
    assert!(usage_text("`nowhere` is no page", false).contains("error: `nowhere` is no page"),);
}

fn text_of(words: &[&str]) -> String {
    let error = toolbox_cli::command()
        .try_get_matches_from(std::iter::once("garuda-toolbox").chain(words.iter().copied()))
        .unwrap_err();

    clap_text(&error.render().to_string())
}

#[test]
fn a_tip_is_a_hint_and_the_usage_block_is_left_out() {
    assert_eq!(
        text_of(&["servics", "list"]),
        "unrecognized subcommand 'servics'\nsome similar subcommands exist: 'users', 'services'"
    );
}

#[test]
fn the_possible_values_are_the_hint() {
    assert_eq!(
        text_of(&["status", "--color", "sometimes"]),
        "invalid value 'sometimes' for '--color <WHEN>'\npossible values: auto, always, never"
    );
}

#[test]
fn listed_arguments_join_the_message_and_the_help_of_the_command_is_the_hint() {
    assert_eq!(
        text_of(&["toggle", "on"]),
        "the following required arguments were not provided: <ID>\ngaruda-toolbox toggle on --help"
    );
}

#[test]
fn an_error_without_a_tip_names_the_help_of_its_command() {
    assert_eq!(
        text_of(&["status", "extra"]),
        "unexpected argument 'extra' found\ngaruda-toolbox status --help"
    );
}

#[test]
fn the_second_line_of_a_usage_is_no_listed_argument() {
    let text = clap_text(
        "error: unrecognized subcommand 'x'\n\nUsage: garuda-toolbox [OPTIONS]\n       garuda-toolbox <COMMAND>\n\nFor more information, try '--help'.\n",
    );

    assert_eq!(text, "unrecognized subcommand 'x'\ngaruda-toolbox --help");
}
