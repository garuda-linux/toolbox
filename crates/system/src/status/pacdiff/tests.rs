use std::path::PathBuf;

use super::{MergeSource as _, Pacdiff, parse_files};
use crate::ProbeError;
use crate::testing::fake_program;

/// `pacdiff -o` as root in the Lima VM.
const PACDIFF: &str = include_str!("fixtures/pacdiff-o.stdout");
const PACDIFF_CODE: &str = include_str!("fixtures/pacdiff-o.code");

fn paths(values: &[&str]) -> Vec<PathBuf> {
    values.iter().map(PathBuf::from).collect()
}

#[test]
fn one_file_per_line() {
    assert_eq!(
        parse_files(PACDIFF),
        paths(&["/etc/locale.gen.pacsave", "/etc/pacman.conf.pacnew"])
    );
}

#[test]
fn empty_output_is_nothing_to_merge() {
    assert_eq!(parse_files(""), paths(&[]));
    assert_eq!(parse_files("\n\n"), paths(&[]));
}

/// Paths may hold spaces. Anything not absolute is a message, not a file.
/// The Lima output with a space put into a name and a made-up message and
/// blank line around it.
#[test]
fn spaces_are_kept_and_messages_skipped() {
    let output = format!(
        "==> some message\n{}  \n",
        PACDIFF.replacen("locale.gen", "locale gen", 1)
    );

    assert_eq!(
        parse_files(&output),
        paths(&["/etc/locale gen.pacsave", "/etc/pacman.conf.pacnew"])
    );
}

#[test]
fn the_adapter_lists_what_pacdiff_prints() {
    let (_folder, fake) = fake_program(PACDIFF, "", PACDIFF_CODE.trim().parse().unwrap());

    assert_eq!(
        Pacdiff::with_program(fake).waiting(),
        Ok(paths(&[
            "/etc/locale.gen.pacsave",
            "/etc/pacman.conf.pacnew"
        ]))
    );
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
