mod root_action_programs;
mod root_actions;

use std::path::Path;

use super::notes::SET_LOCALE_WHY;
use super::{Builtin, Command, DATABASE_LOCK, Invocation, Program};
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

    assert_eq!(
        invocation.to_string(),
        "pacman -Rns $(pacman -Qtdq)  # again until it prints nothing"
    );
}

#[test]
fn an_edit_user_config_describes_itself() {
    let edit = UserConfigEdit::File {
        path: HomePath::new(".config/app.conf").unwrap(),
        content: "on = true\n".parse().unwrap(),
    };

    let builtin = Builtin::EditUserConfig {
        edit,
        on: crate::Toggle::On,
    };

    assert!(builtin.to_string().contains("app.conf"));
}

#[test]
fn the_locale_note_names_the_locale_gen_file() {
    assert_eq!(
        SET_LOCALE_WHY,
        format!("only when /{} has it enabled", crate::locale::LOCALE_GEN)
    );
}

#[test]
fn a_command_leaves_out_the_note_its_display_keeps() {
    let step = Invocation::Builtin(Builtin::RemoveDatabaseLock);

    assert_eq!(step.command(), format!("rm /{DATABASE_LOCK}"));
    assert_eq!(
        step.to_string(),
        format!("rm /{DATABASE_LOCK}  # only while pacman is not running")
    );
}

#[test]
fn unmet_targets_from_a_test_show_as_command_substitution() {
    let invocation = Invocation::RunWithUnmetFrom {
        test: Command::new(Program::Pacman).args(["-T", "--", "cmake"]),
        run: Command::new(Program::Pacman).arg("-S"),
    };

    assert_eq!(
        invocation.to_string(),
        "pacman -S $(pacman -T -- cmake)  # only the packages pacman -T names as missing, so an installed one keeps its install reason"
    );
}
