use super::{ending, push_line};

#[test]
fn ending_names_the_line_break() {
    assert_eq!(ending("en_US.UTF-8 UTF-8"), "");
    assert_eq!(ending("en_US.UTF-8 UTF-8\n"), "\n");
    assert_eq!(ending("en_US.UTF-8 UTF-8\r\n"), "\r\n");
}

#[test]
fn a_pushed_line_ends_the_last_line_first() {
    let mut text = String::new();
    push_line(&mut text, "a");
    assert_eq!(text, "a\n");

    let mut cut = "a".to_owned();
    push_line(&mut cut, "b");
    assert_eq!(cut, "a\nb\n");
}
