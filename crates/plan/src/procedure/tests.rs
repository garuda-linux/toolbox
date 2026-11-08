use std::path::Path;

use super::{Builtin, Command, Invocation, Program};
use crate::{HomePath, UserConfigEdit};

#[test]
fn command_shows_variables_program_name_and_arguments() {
    let command = Command::new(Program::GarudaUpdate)
        .env("GARUDA_UPDATE_TOOLBOX", "1")
        .arg("--skip-mirrorlist");

    assert_eq!(
        command.to_string(),
        "GARUDA_UPDATE_TOOLBOX=1 garuda-update --skip-mirrorlist"
    );
}

#[test]
fn arguments_with_shell_characters_are_quoted_for_display() {
    let command = Command::new(Program::Pacman).args(["-Ss", "a b", "it's"]);
    assert_eq!(command.to_string(), r"pacman -Ss 'a b' 'it'\''s'");
}

#[test]
fn programs_run_from_absolute_paths() {
    assert_eq!(Program::Pacman.path(), "/usr/bin/pacman");
    assert_eq!(Program::PacmanKey.name(), "pacman-key");
    assert_eq!(Program::Journalctl.path(), "/usr/bin/journalctl");
    assert_eq!(Program::Dmesg.path(), "/usr/bin/dmesg");
    assert_eq!(
        Program::Dmesg.below(Path::new("/tmp/root")),
        Path::new("/tmp/root/usr/bin/dmesg")
    );
    assert_eq!(Program::GarudaHealth.path(), "/usr/bin/garuda-health");
}

#[test]
fn targets_from_a_query_show_as_command_substitution() {
    let invocation = Invocation::RunWithTargetsFrom {
        query: Command::new(Program::Pacman).arg("-Qtdq"),
        run: Command::new(Program::Pacman).arg("-Rns"),
    };

    assert_eq!(invocation.to_string(), "pacman -Rns $(pacman -Qtdq)");
}

#[test]
fn an_edit_user_config_describes_itself() {
    let edit = UserConfigEdit::File {
        path: HomePath::new(".config/app.conf").unwrap(),
        content: "on = true\n".to_owned(),
    };

    let builtin = Builtin::EditUserConfig { edit, on: true };

    assert!(builtin.to_string().contains("app.conf"));
}
