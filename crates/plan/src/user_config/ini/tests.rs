use super::{set, value};

const KWINRC: &str = "# kwin\n[Desktops]\nNumber=2\n\n[Windows]\nFocusPolicy=ClickToFocus\nBorderlessMaximizedWindows = false\n\n[Plugins]\nblurEnabled=true\n";

#[test]
fn reads_the_value_of_the_key_in_its_group_only() {
    let text = "[Other]\nBorderlessMaximizedWindows=true\n[Windows]\nBorderlessMaximizedWindows = false \n";

    assert_eq!(
        value(text, "Windows", "BorderlessMaximizedWindows"),
        Some("false")
    );
    assert_eq!(value(text, "Missing", "BorderlessMaximizedWindows"), None);
    assert_eq!(value(text, "Windows", "Borderless"), None);
}

#[test]
fn a_later_entry_wins_and_comments_do_not_count() {
    let text = "[Windows]\nKey=one\n#Key=commented\n[Plugins]\n[Windows]\nKey=two\n";

    assert_eq!(value(text, "Windows", "Key"), Some("two"));
}

#[test]
fn replaces_an_existing_entry_and_keeps_everything_else() {
    let after = set(KWINRC, "Windows", "BorderlessMaximizedWindows", "true");

    assert_eq!(
        after,
        KWINRC.replace(
            "BorderlessMaximizedWindows = false",
            "BorderlessMaximizedWindows=true"
        )
    );
}

#[test]
fn adds_the_key_after_the_last_line_of_its_group() {
    let text = "[Windows]\nFocusPolicy=ClickToFocus\n\n[Plugins]\nblurEnabled=true\n";

    assert_eq!(
        set(text, "Windows", "BorderlessMaximizedWindows", "true"),
        "[Windows]\nFocusPolicy=ClickToFocus\nBorderlessMaximizedWindows=true\n\n[Plugins]\nblurEnabled=true\n"
    );
}

#[test]
fn adds_the_group_when_it_is_missing() {
    assert_eq!(
        set("[Plugins]\nblurEnabled=true", "Windows", "Key", "true"),
        "[Plugins]\nblurEnabled=true\n\n[Windows]\nKey=true\n"
    );
    assert_eq!(set("", "Windows", "Key", "true"), "[Windows]\nKey=true\n");
}

#[test]
fn a_last_line_without_newline_gets_one_before_the_new_entry() {
    assert_eq!(
        set("[Windows]\nFocusPolicy=Click", "Windows", "Key", "true"),
        "[Windows]\nFocusPolicy=Click\nKey=true\n"
    );
}

#[test]
fn keeps_crlf_line_endings_of_replaced_lines() {
    assert_eq!(
        set(
            "[Windows]\r\nKey=false\r\nOther=1\r\n",
            "Windows",
            "Key",
            "true"
        ),
        "[Windows]\r\nKey=true\r\nOther=1\r\n"
    );
}

#[test]
fn setting_the_current_value_changes_nothing() {
    let text = "[Windows]\nKey=true\n";

    assert_eq!(set(text, "Windows", "Key", "true"), text);
}
