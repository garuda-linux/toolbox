use super::{Choice, Part, Plural, Value, messages};

fn one(source: &str) -> Result<Value, String> {
    let mut found = messages(source).expect("the source parses");
    assert_eq!(found.len(), 1, "one message");
    found.remove(0).value
}

fn text(value: &str) -> Part {
    Part::Text(value.to_owned())
}

fn var(name: &str) -> Part {
    Part::Var(name.to_owned())
}

#[test]
fn a_plain_message_is_text_and_variables() {
    assert_eq!(
        one("greet = Hello, { $user }!\n"),
        Ok(Value::Plain(vec![text("Hello, "), var("user"), text("!")]))
    );
}

#[test]
fn a_string_literal_is_its_text() {
    assert_eq!(one("sep = {\", \"}\n"), Ok(Value::Plain(vec![text(", ")])));
}

#[test]
fn a_multiline_value_keeps_its_line_breaks() {
    assert_eq!(
        one("long =\n    first line\n    second line\n"),
        Ok(Value::Plain(vec![text("first line\nsecond line")]))
    );
}

#[test]
fn a_plural_choice_is_a_plural_inside_its_text() {
    let source =
        "count = Found { $n ->\n        [one] one file\n       *[other] { $n } files\n    } here\n";

    assert_eq!(
        one(source),
        Ok(Value::Plain(vec![
            text("Found "),
            Part::Plural(Plural {
                var: "n".to_owned(),
                variants: vec![
                    ("one".to_owned(), vec![text("one file")]),
                    ("other".to_owned(), vec![var("n"), text(" files")]),
                ],
            }),
            text(" here"),
        ]))
    );
}

#[test]
fn a_whole_message_choice_on_other_words_is_a_choice() {
    let source = "ask = { $subject ->\n    [report] Upload this report?\n   *[output] Upload this output?\n}\n";

    assert_eq!(
        one(source),
        Ok(Value::Choice(Choice {
            var: "subject".to_owned(),
            default: "output".to_owned(),
            variants: vec![
                ("report".to_owned(), vec![text("Upload this report?")]),
                ("output".to_owned(), vec![text("Upload this output?")]),
            ],
        }))
    );
}

#[test]
fn the_comment_above_a_message_is_its_note() {
    let found = messages("# Shown on the button.\nsave = Save\n").expect("parses");

    assert_eq!(found[0].note.as_deref(), Some("Shown on the button."));
}

#[test]
fn what_cannot_round_trip_says_why() {
    for (source, reason) in [
        ("f = { NUMBER($n) }\n", "function"),
        ("r = { other-message }\n", "another message"),
        ("t = { -brand }\n", "term"),
        ("k = { $n ->\n    [1] one\n   *[other] more\n}\n", "number"),
        (
            "c = Pick { $s ->\n    [a] a\n   *[b] b\n} now\n",
            "choice inside",
        ),
        ("a = Value\n    .title = Title\n", "attribute"),
        (
            "n = { $a ->\n    [one] { $b ->\n        [one] x\n       *[other] y\n    }\n   *[other] z\n}\n",
            "nested",
        ),
    ] {
        let reason_found = one(source).expect_err(source);
        assert!(
            reason_found.contains(reason),
            "`{source}` gave `{reason_found}`, expected it to mention `{reason}`"
        );
    }
}
