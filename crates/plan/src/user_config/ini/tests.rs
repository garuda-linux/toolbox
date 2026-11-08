use super::{set, value};

/// `~/.config/kwinrc` after `kwriteconfig6` set one key in the Lima image.
const WRITTEN: &str = include_str!("../fixtures/kwinrc");
/// `~/.config/kwinrc` of a used Plasma session, nested `[Tiling]` groups
/// and all.
const USED: &str = include_str!("../fixtures/installed/kwinrc");

const KEY: &str = "BorderlessMaximizedWindows";

#[test]
fn reads_the_value_of_the_key_in_its_group_only() {
    assert_eq!(value(USED, "Windows", KEY), Some("true"));
    assert_eq!(value(USED, "Plugins", "scaleEnabled"), Some("false"));
    assert_eq!(value(USED, "Xwayland", "Scale"), Some("1"));
    assert_eq!(
        value(USED, "org.kde.kdecoration2", "ButtonsOnRight"),
        Some("")
    );
    assert_eq!(value(USED, "Plugins", KEY), None);
    assert_eq!(value(USED, "Missing", KEY), None);
    assert_eq!(value(USED, "Windows", "Borderless"), None);

    // Edit: blanks around the `=` of the real line.
    let spaced = USED.replace(
        "BorderlessMaximizedWindows=true",
        "BorderlessMaximizedWindows = false ",
    );

    assert_eq!(value(&spaced, "Windows", KEY), Some("false"));
}

#[test]
fn a_later_entry_wins_and_comments_do_not_count() {
    // Edit: a second `[Windows]` group with the key commented and set.
    let text = format!("{USED}\n[Windows]\n#{KEY}=commented\n{KEY}=two\n");

    assert_eq!(value(&text, "Windows", KEY), Some("two"));
}

#[test]
fn replaces_an_existing_entry_and_keeps_everything_else() {
    let after = set(USED, "Windows", KEY, "false");

    assert_eq!(after, USED.replace("Windows=true", "Windows=false"));

    // Edit: blanks around the `=` of the real line.
    let spaced = WRITTEN.replace("Windows=true", "Windows = true");

    assert_eq!(
        set(&spaced, "Windows", KEY, "false"),
        WRITTEN.replace("true", "false")
    );
}

#[test]
fn adds_the_key_after_the_last_line_of_its_group() {
    assert_eq!(
        set(USED, "Plugins", "zoomEnabled", "true"),
        USED.replace(
            "wobblywindowsEnabled=true\n",
            "wobblywindowsEnabled=true\nzoomEnabled=true\n"
        )
    );
}

#[test]
fn adds_the_group_when_it_is_missing() {
    assert_eq!(
        set(WRITTEN, "Plugins", "blurEnabled", "true"),
        format!("{WRITTEN}\n[Plugins]\nblurEnabled=true\n")
    );

    // Edit: the real file without its last line break.
    assert_eq!(
        set(WRITTEN.trim_end(), "Plugins", "blurEnabled", "true"),
        format!("{WRITTEN}\n[Plugins]\nblurEnabled=true\n")
    );
    assert_eq!(set("", "Windows", KEY, "true"), WRITTEN);
}

#[test]
fn a_last_line_without_newline_gets_one_before_the_new_entry() {
    // Edit: the real file without its last line break.
    assert_eq!(
        set(WRITTEN.trim_end(), "Windows", "FocusPolicy", "ClickToFocus"),
        format!("{WRITTEN}FocusPolicy=ClickToFocus\n")
    );
}

#[test]
fn keeps_crlf_line_endings_of_replaced_lines() {
    // Edit: the real file with CRLF endings.
    let crlf = WRITTEN.replace('\n', "\r\n");

    assert_eq!(
        set(&crlf, "Windows", KEY, "false"),
        crlf.replace("true", "false")
    );
}

#[test]
fn setting_the_current_value_changes_nothing() {
    assert_eq!(set(USED, "Windows", KEY, "true"), USED);
    assert_eq!(set(WRITTEN, "Windows", KEY, "true"), WRITTEN);
}
