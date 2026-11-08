use std::ffi::OsStr;
use std::path::Path;

use super::PtyCommand;
use crate::{DEFAULT_SIZE, TerminalSize};

fn env_of<'command>(
    command: &'command std::process::Command,
    key: &str,
) -> Option<&'command OsStr> {
    let (_, value) = command
        .get_envs()
        .find(|(name, _)| *name == OsStr::new(key))?;

    value
}

#[test]
fn a_command_carries_program_and_arguments() {
    let command = PtyCommand::new("/usr/bin/pacman")
        .arg("-S")
        .args(["--needed", "vlc"]);

    let built = command.to_command();
    assert_eq!(built.get_program(), "/usr/bin/pacman");
    assert_eq!(
        built.get_args().collect::<Vec<_>>(),
        ["-S", "--needed", "vlc"]
    );
    assert_eq!(command.program(), Path::new("/usr/bin/pacman"));
}

#[test]
fn term_is_always_xterm_256color() {
    let command = PtyCommand::new("pacman")
        .env("TERM", "dumb")
        .env("PATH", "/usr/bin");

    let built = command.to_command();
    assert_eq!(env_of(&built, "TERM"), Some(OsStr::new("xterm-256color")));
    assert_eq!(env_of(&built, "PATH"), Some(OsStr::new("/usr/bin")));
}

#[test]
fn the_size_is_clamped_and_defaults_to_the_default_size() {
    assert_eq!(PtyCommand::new("pacman").terminal_size(), DEFAULT_SIZE);

    let tiny = PtyCommand::new("pacman").size(TerminalSize {
        columns: 1,
        rows: 1,
    });

    assert_eq!(
        tiny.terminal_size(),
        TerminalSize {
            columns: 40,
            rows: 10
        }
    );
}

#[test]
fn a_clean_environment_keeps_only_what_was_set() {
    let command = PtyCommand::new("pacman").env_clear().env("LANG", "C.UTF-8");
    let built = command.to_command();
    let names: Vec<&OsStr> = built.get_envs().map(|(name, _)| name).collect();
    assert_eq!(names, [OsStr::new("LANG"), OsStr::new("TERM")]);
}
