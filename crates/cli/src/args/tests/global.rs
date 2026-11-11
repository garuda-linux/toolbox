use clap::error::ErrorKind;

use super::super::{Command, ToggleCommand};
use super::parse;
use crate::output::ColorMode;

#[test]
fn output_flags_go_before_the_command_too() {
    let cli = parse(&["--color", "never", "--no-pager", "-q", "status"]).unwrap();

    assert!(matches!(cli.command, Some(Command::Status(_))));
    assert_eq!(cli.color, ColorMode::Never);
    assert!(cli.no_pager);
    assert!(cli.quiet);
}

#[test]
fn output_flags_go_after_the_command_as_before() {
    let cli = parse(&["toggle", "list", "--color", "never", "--width", "60"]).unwrap();

    assert!(matches!(
        cli.command,
        Some(Command::Toggle(ToggleCommand::List(_)))
    ));
    assert_eq!(cli.color, ColorMode::Never);
    assert_eq!(cli.width, Some(60));
}

#[test]
fn a_window_flag_never_goes_with_a_command() {
    for words in [
        &["--page", "maintenance", "status"][..],
        &["--setup-assistant", "doctor"],
        &["--boot-repair", "pages"],
    ] {
        let error = parse(words).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ArgumentConflict, "{words:?}");
    }
}

#[test]
fn a_window_flag_alone_still_opens_the_window() {
    let cli = parse(&["--color", "never", "--page", "maintenance"]).unwrap();

    assert!(cli.command.is_none());
    assert!(cli.window.page.is_some());
}
