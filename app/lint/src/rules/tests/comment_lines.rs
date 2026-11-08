use std::fmt::Write as _;

use super::count;

const RULE: &str = "max-comment-lines";

fn lines(prefix: &str, n: usize) -> String {
    (0..n).fold(String::new(), |mut text, i| {
        writeln!(text, "{prefix} line {i}").unwrap();
        text
    })
}

#[test]
fn body_comments_allow_three_lines() {
    let body = |n| format!("fn f() {{\n{}    g();\n}}\n", lines("    //", n));
    assert_eq!(count(RULE, &body(3)), 0);
    assert_eq!(count(RULE, &body(4)), 1);
}

#[test]
fn other_comments_allow_four_lines() {
    assert_eq!(count(RULE, &format!("{}fn f() {{}}\n", lines("//", 4))), 0);
    assert_eq!(count(RULE, &format!("{}fn f() {{}}\n", lines("//", 5))), 1);
}

#[test]
fn doc_descriptions_allow_six_lines_before_sections() {
    let doc = |n| {
        format!(
            "{}///\n/// # Errors\n///\n/// a\n/// b\n/// c\nfn f() {{}}\n",
            lines("///", n)
        )
    };

    assert_eq!(count(RULE, &doc(6)), 0);
    assert_eq!(count(RULE, &doc(7)), 1);
    assert_eq!(count(RULE, &lines("//!", 20)), 0);
}

#[test]
fn blank_lines_in_doc_descriptions_do_not_count() {
    let text = format!("/// Summary.\n///\n{}fn f() {{}}\n", lines("///", 5));
    assert_eq!(count(RULE, &text), 0);
}
