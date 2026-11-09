//! The asked path passed the write table when the request was decoded,
//! but a write goes where its links lead. That real path must pass the
//! table too, or a link in a writable folder reaches any file the toolbox
//! only shows.

use std::fs;
use std::os::unix::fs::symlink;

use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{ConfigWrite, Fingerprint, StepId};

use super::super::super::tests::{BACKUPS, path, text};
use super::super::{WriteError, execute};
use crate::testing::printed;

const OLD: &str = "root ALL=(ALL) ALL\n";

#[test]
fn a_link_from_a_writable_folder_to_a_read_only_file_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let sudoers = root.path().join("etc/sudoers");

    fs::create_dir_all(root.path().join("etc/modprobe.d")).unwrap();
    fs::write(&sudoers, OLD).unwrap();
    symlink("../sudoers", root.path().join("etc/modprobe.d/x.conf")).unwrap();

    let write = ConfigWrite {
        path: path("/etc/modprobe.d/x.conf"),
        content: text("ALL ALL=(ALL) NOPASSWD: ALL\n"),
        original: Fingerprint::of(OLD.as_bytes()),
    };

    let mut events = Vec::new();

    assert_eq!(
        execute(root.path(), StepId::new(1), &write, &mut events),
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );

    let shown = printed(&events);

    assert!(
        shown.contains("/etc/sudoers (where the link /etc/modprobe.d/x.conf leads) cannot be replaced: the toolbox never writes /etc/sudoers"),
        "{shown}"
    );
    assert_eq!(fs::read_to_string(&sudoers).unwrap(), OLD);
    assert!(!root.path().join(BACKUPS).exists(), "a backup was written");
}

#[test]
fn a_refused_real_path_names_the_file_that_stays() {
    let error = WriteError::NotWritable {
        real: path("/etc/sudoers"),
    };

    assert_eq!(error.to_string(), "the toolbox never writes /etc/sudoers");
}
