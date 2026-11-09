//! The chosen disk is not trusted: what it holds must neither act as a
//! device or setuid program on the live system, nor bend a cleanup path
//! through its own symlinks onto the live system's mounts.

use super::{always_command, every_run, run_command};
use crate::procedure::{Command, Program};

fn from_the_disk(command: &Command) -> bool {
    let arguments = command.arguments();

    command.program() == Program::Mount
        && !arguments.iter().any(|argument| argument == "--bind")
        && !arguments.iter().any(|argument| argument == "efivarfs")
}

#[test]
fn every_file_system_of_the_disk_is_mounted_without_devices_and_setuid() {
    for (action, steps) in every_run() {
        for command in steps
            .iter()
            .filter_map(run_command)
            .filter(|command| from_the_disk(command))
        {
            let options = command
                .arguments()
                .iter()
                .skip_while(|argument| *argument != "-o")
                .nth(1);

            assert!(
                options.is_some_and(|options| options.starts_with("nodev,nosuid")),
                "{command} in {action:?}"
            );
        }
    }
}

/// A disk whose `dev` is a symlink to `/dev` must not make the cleanup
/// unmount the live system's `/dev`.
#[test]
fn every_unmount_takes_its_path_as_written() {
    for (action, steps) in every_run() {
        for command in steps
            .iter()
            .filter_map(always_command)
            .filter(|command| command.program() == Program::Umount)
        {
            assert!(
                command
                    .arguments()
                    .iter()
                    .any(|argument| argument == "--no-canonicalize"),
                "{command} in {action:?}"
            );
        }
    }
}
