use tree_sitter::Point;

use super::{Kind, Source, looks_like_code, parse};

fn comments(text: &str) -> Vec<(Kind, Vec<String>, Point, bool)> {
    let tree = parse(text).unwrap();
    let source = Source::new(text, &tree);
    source
        .comments
        .iter()
        .map(|comment| {
            (
                comment.kind,
                comment.lines.iter().map(|line| line.text.clone()).collect(),
                comment.lines[0].at,
                comment.alone,
            )
        })
        .collect()
}

#[test]
fn rust_comments_lose_their_markers() {
    let found =
        comments("//! Module.\n/// Item.\nfn f() { g(); // trailing\n}\n/*\n * Block\n */\n");

    assert_eq!(
        found[0],
        (
            Kind::InnerDoc,
            vec!["Module.".to_owned()],
            Point::new(0, 4),
            true
        )
    );
    assert_eq!(
        found[1],
        (Kind::Doc, vec!["Item.".to_owned()], Point::new(1, 4), true)
    );
    assert_eq!(
        found[2],
        (
            Kind::Line,
            vec!["trailing".to_owned()],
            Point::new(2, 17),
            false
        )
    );
    assert_eq!(
        found[3],
        (
            Kind::Block,
            vec!["Block".to_owned()],
            Point::new(5, 3),
            true
        )
    );
}

#[test]
fn code_is_told_from_prose() {
    assert!(looks_like_code("let x = foo(1);"));
    assert!(looks_like_code("x.y = 3"));
    for prose in ["Read the file", "Done", "Note: the width", "TODO: later"] {
        assert!(!looks_like_code(prose), "{prose}");
    }
}
