use toolbox_cli::output::Style;

use super::{long_help, named};

#[test]
fn coloured_help_keeps_its_escape_codes() {
    let command = toolbox_cli::command();

    assert!(long_help(&command, Style::Color).contains("\u{1b}["));
}

fn plain_help_of(words: &[&str]) -> String {
    let words: Vec<String> = words.iter().map(|word| (*word).to_owned()).collect();
    let command = named(&words).expect("a command");

    long_help(&command, Style::Plain)
}

#[test]
fn a_subcommand_help_names_the_whole_command() {
    let text = plain_help_of(&["toggle", "on"]);

    assert!(text.contains("Usage: garuda-toolbox toggle on "), "{text}");
}

#[test]
fn a_subcommand_help_has_the_global_flags_and_examples() {
    let text = plain_help_of(&["toggle"]);

    assert!(text.contains("--no-pager"), "{text}");
    assert!(text.contains("Examples:"), "{text}");
}

#[test]
fn a_word_that_is_no_command_finds_nothing() {
    assert_eq!(named(&["nosuch".to_owned()]).unwrap_err(), "nosuch");
}

#[test]
fn plain_help_has_none() {
    let command = toolbox_cli::command();
    let text = long_help(&command, Style::Plain);

    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("Usage"));
}
