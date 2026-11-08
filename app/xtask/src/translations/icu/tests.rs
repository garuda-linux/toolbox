use super::{icu, parse};
use crate::translations::model::{Part, Plural};

fn text(value: &str) -> Part {
    Part::Text(value.to_owned())
}

fn var(name: &str) -> Part {
    Part::Var(name.to_owned())
}

fn steps() -> Vec<Part> {
    vec![
        text("Applied, "),
        Part::Plural(Plural {
            var: "count".to_owned(),
            variants: vec![
                ("one".to_owned(), vec![text("1 step")]),
                ("other".to_owned(), vec![var("count"), text(" steps")]),
            ],
        }),
    ]
}

#[test]
fn variables_become_icu_arguments() {
    let parts = vec![text("Hello, "), var("user"), text("!")];

    assert_eq!(icu(&parts), "Hello, {user}!");
    assert_eq!(parse("Hello, {user}!"), Ok(parts));
}

#[test]
fn a_plural_becomes_an_icu_plural() {
    let written = icu(&steps());

    assert_eq!(
        written,
        "Applied, {count, plural, one {1 step} other {{count} steps}}"
    );
    assert_eq!(parse(&written), Ok(steps()));
}

#[test]
fn literal_braces_and_apostrophes_survive() {
    for raw in [
        "a { brace }",
        "it's fine",
        "quote' before",
        "end'",
        "''double",
    ] {
        let parts = vec![text(raw)];
        assert_eq!(parse(&icu(&parts)), Ok(parts), "`{raw}`");
    }

    let before_variable = vec![text("name'"), var("x")];
    assert_eq!(parse(&icu(&before_variable)), Ok(before_variable));
}

#[test]
fn a_hash_is_literal_outside_a_plural_and_the_number_inside() {
    assert_eq!(parse("issue #5"), Ok(vec![text("issue #5")]));

    let inside = "{n, plural, one {# file} other {# files '#'1}}";
    let Ok(parts) = parse(inside) else {
        panic!("parses");
    };

    assert_eq!(
        parts,
        vec![Part::Plural(Plural {
            var: "n".to_owned(),
            variants: vec![
                ("one".to_owned(), vec![var("n"), text(" file")]),
                ("other".to_owned(), vec![var("n"), text(" files #1")]),
            ],
        })]
    );
}

#[test]
fn what_fluent_cannot_hold_is_refused() {
    for (written, reason) in [
        ("{s, select, a {x} other {y}}", "select"),
        ("{n, plural, offset:1 one {x} other {y}}", "offset"),
        ("{n, plural, =1 {x} other {y}}", "exact"),
        ("{n, plural, one {x}}", "other"),
        ("open {brace", "closed"),
        ("close } brace", "unbalanced"),
        ("{n, number}", "number"),
    ] {
        let found = parse(written).expect_err(written);
        assert!(
            found.contains(reason),
            "`{written}` gave `{found}`, expected `{reason}`"
        );
    }
}
