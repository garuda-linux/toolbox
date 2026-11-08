use super::{ESCAPE, strip_ansi};

/// What `script(1)` caught of pacman asking about a conflict and about a
/// provider, colours, cursor codes and carriage returns included.
const CONFLICT: &str = include_str!("../../tests/fixtures/pacman-conflict.txt");
const PROVIDER: &str = include_str!("../../tests/fixtures/pacman-provider.txt");

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

#[test]
fn pacman_s_questions_read_as_plain_text() {
    let conflict = strip_ansi(CONFLICT);
    let provider = strip_ansi(PROVIDER);

    for plain in [&conflict, &provider] {
        assert!(!plain.contains(ESCAPE), "{plain:?}");
    }

    let conflict_lines: Vec<&str> = conflict.lines().collect();

    assert!(
        conflict_lines.contains(
            &":: vlc-git-4.0.0.r39705.g978212a-1 and vlc-3.0.24-1 are in conflict. \
              Remove vlc? [y/N] error: unresolvable package conflicts detected"
        ),
        "{conflict_lines:#?}"
    );

    let provider_lines: Vec<&str> = provider.lines().collect();

    assert_eq!(
        provider_lines.get(2..6).unwrap(),
        [
            ":: There are 19 providers available for java-runtime:",
            ":: Repository extra",
            "   1) jdk-openjdk  2) jdk11-openjdk  3) jdk17-openjdk  4) jdk21-openjdk",
            "   5) jdk25-openjdk  6) jre-openjdk  7) jre11-openjdk  8) jre17-openjdk",
        ]
    );
    assert!(provider.contains("\r\nEnter a number (default=1): resolving dependencies..."));
    assert!(provider.ends_with("\r\n:: Proceed with installation? [Y/n] "));
}
