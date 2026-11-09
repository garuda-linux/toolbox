use super::{ending, entries, lossy, push_line};

/// `/etc/passwd` of the Lima image, its user renamed to `user`.
const PASSWD: &str = include_str!("../../../system/src/accounts/fixtures/passwd");

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

#[test]
fn valid_text_stays_as_it_is() {
    assert_eq!(lossy(PASSWD.as_bytes().to_vec()), PASSWD);
}

/// Edit: root's empty full name replaced by the Latin-1 byte `\xFF`.
#[test]
fn a_byte_that_is_no_utf8_changes_only_itself() {
    let (root, rest) = PASSWD.split_once("::/root").unwrap();
    let text = lossy([root.as_bytes(), b":\xFF:/root", rest.as_bytes()].concat());

    assert_eq!(text.lines().count(), PASSWD.lines().count());
    assert!(
        text.starts_with("root:x:0:0:\u{fffd}:/root:/usr/bin/bash\n"),
        "{text}"
    );
}

/// Edit: a comment, a blank line and blanks before root's entry added.
#[test]
fn entries_skip_comments_and_blank_lines() {
    let edited = format!("# accounts\n\n  {PASSWD}");
    let lines: Vec<&str> = entries(&edited).collect();

    assert_eq!(lines.len(), PASSWD.lines().count());
    assert_eq!(lines.first(), Some(&"root:x:0:0::/root:/usr/bin/bash"));
}
