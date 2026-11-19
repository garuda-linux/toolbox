use std::path::PathBuf;

use toolbox_plan::MergeCopy;

use super::{MergeSource as _, Pacdiff, merge_copies, parse_files};
use crate::ProbeError;
use crate::test_support::fake_program;

/// `pacdiff -o` as root in the Lima VM.
const PACDIFF: &str = include_str!("fixtures/pacdiff-o.stdout");
const PACDIFF_CODE: &str = include_str!("fixtures/pacdiff-o.code");

struct Listed(Vec<PathBuf>);

impl super::MergeSource for Listed {
    fn waiting(&self) -> Result<Vec<PathBuf>, ProbeError> {
        Ok(self.0.clone())
    }
}

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
        Err(error) if error.is_failure()
    ));
}

#[test]
fn a_silent_pacdiff_has_nothing_to_merge() {
    assert_eq!(Pacdiff::with_program("true").waiting(), Ok(Vec::new()));
}

#[test]
fn the_snapshot_holds_each_copy_the_toolbox_may_open() {
    let copies = merge_copies(&Listed(parse_files(PACDIFF))).unwrap();
    let names: Vec<&str> = copies.0.iter().map(MergeCopy::as_str).collect();

    assert_eq!(
        names,
        ["/etc/locale.gen.pacsave", "/etc/pacman.conf.pacnew"]
    );
}

/// The Lima output with a copy of the password hashes added.
#[test]
fn a_copy_of_a_withheld_file_stays_out_of_the_snapshot() {
    let output = format!("{PACDIFF}/etc/shadow.pacnew\n");
    let copies = merge_copies(&Listed(parse_files(&output))).unwrap();

    assert_eq!(copies.0.len(), 2);
}

#[test]
fn a_self_test_finds_the_copies_among_its_own_files() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");

    std::fs::create_dir_all(&etc).unwrap();
    for name in ["pacman.conf", "pacman.conf.pacnew", "locale.gen.pacsave"] {
        std::fs::write(etc.join(name), "").unwrap();
    }

    assert_eq!(
        super::FoundCopies::new(root.path().to_owned()).waiting(),
        Ok(paths(&[
            "/etc/locale.gen.pacsave",
            "/etc/pacman.conf.pacnew"
        ]))
    );
}
