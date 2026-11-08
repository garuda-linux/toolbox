use toolbox_cli::Exit;

use super::{clap_said, json_asked, usage_text};
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
