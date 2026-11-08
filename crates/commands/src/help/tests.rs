use toolbox_cli::output::Style;

use super::long_help;

#[test]
fn coloured_help_keeps_its_escape_codes() {
    let command = toolbox_cli::command();

    assert!(long_help(&command, Style::Color).contains("\u{1b}["));
}

#[test]
fn plain_help_has_none() {
    let command = toolbox_cli::command();
    let text = long_help(&command, Style::Plain);

    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("Usage"));
}
