use super::{Edit, apply, render};
use crate::translations::model::{Choice, Part, Plural, Value, messages};

fn text(value: &str) -> Part {
    Part::Text(value.to_owned())
}

fn var(name: &str) -> Part {
    Part::Var(name.to_owned())
}

/// The rendered message parses back to the same value.
fn round_trip(value: &Value) {
    let source = format!("{}\n", render("id", value));
    let parsed = messages(&source).expect(&source);

    assert_eq!(parsed.len(), 1, "{source}");
    assert_eq!(parsed[0].value.as_ref(), Ok(value), "{source}");
}

#[test]
fn a_plain_value_is_one_line() {
    let value = Value::Plain(vec![text("Hallo, "), var("user"), text("!")]);

    assert_eq!(render("greet", &value), "greet = Hallo, { $user }!");
    round_trip(&value);
}

#[test]
fn what_fluent_reads_as_syntax_becomes_a_literal() {
    for raw in [
        "a { brace }",
        "[bracket first",
        "*star first",
        ".dot first",
        " leading space",
        "trailing space ",
        ", ",
        "back\\slash and \"quote\"",
    ] {
        round_trip(&Value::Plain(vec![text(raw)]));
    }
}

#[test]
fn line_breaks_make_a_block() {
    let value = Value::Plain(vec![text("first\n[second\nthird")]);

    assert_eq!(
        render("long", &value),
        "long =\n    first\n    {\"[\"}second\n    third"
    );
    round_trip(&value);
}

#[test]
fn a_plural_is_written_as_a_block() {
    let value = Value::Plain(vec![
        text("Übernommen, "),
        Part::Plural(Plural {
            var: "count".to_owned(),
            variants: vec![
                ("one".to_owned(), vec![text("1 Schritt")]),
                ("other".to_owned(), vec![var("count"), text(" Schritte")]),
            ],
        }),
    ]);

    assert_eq!(
        render("partly", &value),
        "partly =\n    Übernommen, { $count ->\n        [one] 1 Schritt\n       *[other] { $count } Schritte\n    }"
    );
    round_trip(&value);
}

#[test]
fn a_choice_keeps_its_default_key() {
    let value = Value::Choice(Choice {
        var: "subject".to_owned(),
        default: "output".to_owned(),
        variants: vec![
            ("report".to_owned(), vec![text("Diesen Bericht hochladen?")]),
            ("output".to_owned(), vec![text("Diese Ausgabe hochladen?")]),
        ],
    });

    round_trip(&value);
    assert!(render("ask", &value).contains("       *[output] "));
}

#[test]
fn an_edit_replaces_one_message_and_its_mark_and_keeps_the_rest() {
    let file = "# Header\n\n# A note.\nsave = Save\n# untranslated\nopen = Open\nlong =\n    { $n ->\n        [one] x\n       *[other] y\n    }\nlast = Last\n";
    let edits = [
        Edit::Replace {
            id: "open".to_owned(),
            lines: "open = Öffnen".to_owned(),
        },
        Edit::Replace {
            id: "long".to_owned(),
            lines: "long = Lang".to_owned(),
        },
    ];

    assert_eq!(
        apply(file, &edits),
        "# Header\n\n# A note.\nsave = Save\nopen = Öffnen\nlong = Lang\nlast = Last\n"
    );
}

#[test]
fn an_insert_follows_its_anchor_or_ends_the_file() {
    let file = "a = A\nc = C\n";
    let edits = [
        Edit::Insert {
            after: Some("a".to_owned()),
            lines: "b = B".to_owned(),
        },
        Edit::Insert {
            after: None,
            lines: "z = Z".to_owned(),
        },
    ];

    assert_eq!(apply(file, &edits), "a = A\nb = B\nc = C\nz = Z\n");
}
