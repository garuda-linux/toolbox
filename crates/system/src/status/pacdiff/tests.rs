use std::path::PathBuf;

use super::{MergeSource as _, Pacdiff, parse_files};
use crate::ProbeError;

fn paths(values: &[&str]) -> Vec<PathBuf> {
    values.iter().map(PathBuf::from).collect()
}

#[test]
fn one_file_per_line() {
    let output = "/etc/pacman.conf.pacnew\n\
                  /etc/mkinitcpio.conf.pacsave\n\
                  /etc/locale.gen.pacsave.1\n\
                  /etc/sudoers.pacorig\n";

    assert_eq!(
        parse_files(output),
        paths(&[
            "/etc/pacman.conf.pacnew",
            "/etc/mkinitcpio.conf.pacsave",
            "/etc/locale.gen.pacsave.1",
            "/etc/sudoers.pacorig",
        ])
    );
}

#[test]
fn empty_output_is_nothing_to_merge() {
    assert_eq!(parse_files(""), paths(&[]));
    assert_eq!(parse_files("\n\n"), paths(&[]));
}

/// Paths may hold spaces. Anything not absolute is a message, not a file.
#[test]
fn spaces_are_kept_and_messages_skipped() {
    let output = "==> some message\n/etc/odd name.conf.pacnew\n  \n";

    assert_eq!(parse_files(output), paths(&["/etc/odd name.conf.pacnew"]));
}

#[test]
fn a_missing_pacdiff_is_unavailable() {
    let pacdiff = Pacdiff::with_program("toolbox-system-no-such-program");

    assert_eq!(pacdiff.waiting(), Err(ProbeError::Unavailable));
}

#[test]
fn a_failing_pacdiff_fails() {
    assert!(matches!(
        Pacdiff::with_program("false").waiting(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn a_silent_pacdiff_has_nothing_to_merge() {
    assert_eq!(Pacdiff::with_program("true").waiting(), Ok(Vec::new()));
}
