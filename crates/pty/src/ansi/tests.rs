use super::strip_ansi;

#[test]
fn ansi_is_stripped() {
    assert_eq!(strip_ansi("\u{1b}[1;32mok\u{1b}[0m done"), "ok done");
    assert_eq!(strip_ansi("\u{1b}]0;title\u{7}text"), "text");
    assert_eq!(strip_ansi("plain"), "plain");
    assert_eq!(strip_ansi("\u{1b}"), "");
}

#[test]
fn stripped_text_holds_no_control_characters() {
    for (text, stripped) in [
        ("ok\u{7}bell", "okbell"),
        ("a\u{8}b", "ab"),
        ("a\u{0}b", "ab"),
        ("a\u{84}b", "ab"),
        ("tab\there\r\n", "tab\there\r\n"),
        (
            "family \u{1f468}\u{200d}\u{1f469}",
            "family \u{1f468}\u{200d}\u{1f469}",
        ),
    ] {
        assert_eq!(strip_ansi(text), stripped, "{text:?}");
    }
}

#[test]
fn osc_and_other_escapes_are_stripped() {
    for (text, plain) in [
        ("\u{1b}]8;;https://x\u{1b}\\link\u{1b}]8;;\u{1b}\\", "link"),
        // `tput sgr0` writes `ESC ( B` before the colour reset.
        ("\u{1b}(B\u{1b}[mplain", "plain"),
        ("\u{1b}7saved\u{1b}8", "saved"),
        ("\u{1b}]0;never ends", ""),
        ("a\u{1b}[", "a"),
    ] {
        assert_eq!(strip_ansi(text), plain, "{text:?}");
    }
}
