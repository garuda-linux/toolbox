use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::{ConfigFilePath, ConfigText, MergeCopy};

use super::read_pair;
use crate::config_files::{HelperFetch, SystemConfigFiles};

/// The files are the user's to read, so the helper is never asked.
struct NoHelper;

impl HelperFetch for NoHelper {
    fn fetch_command(&self, _path: &ConfigFilePath) -> Command {
        Command::new("/bin/false")
    }
}

fn text(content: &str) -> ConfigText {
    ConfigText::new(content).unwrap()
}

#[test]
fn both_files_of_a_copy_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");

    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(root.join("etc/pacman.conf"), "#Color\n").unwrap();
    fs::write(root.join("etc/pacman.conf.pacnew"), "Color\n").unwrap();

    let source = SystemConfigFiles::new(&root, dir.path().join("home"), NoHelper);
    let pair = read_pair(
        &source,
        &MergeCopy::new("/etc/pacman.conf.pacnew").unwrap(),
        &Arc::new(AtomicBool::new(false)),
    );

    assert_eq!(pair.copied, Ok(text("Color\n")));
    assert_eq!(pair.current, Ok(Some(text("#Color\n"))));
}

#[test]
fn a_missing_file_in_use_is_no_problem() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");

    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(root.join("etc/locale.gen.pacsave"), "en_US.UTF-8 UTF-8\n").unwrap();

    let source = SystemConfigFiles::new(&root, dir.path().join("home"), NoHelper);
    let pair = read_pair(
        &source,
        &MergeCopy::new("/etc/locale.gen.pacsave").unwrap(),
        &Arc::new(AtomicBool::new(false)),
    );

    assert_eq!(pair.current, Ok(None));
}
