//! Merges against the image's `/etc/pacman.conf`, a pacman backup file.
//! Each test restores both files.

use std::error::Error;
use std::fs;
use std::path::Path;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{
    ConfigFilePath, ConfigText, FetchAnswer, Fingerprint, MergeChoice, MergeCopy, MergeResolve,
    Resolution, RootAction,
};

use super::common::{TestResult, helper, request};
use super::{apply, in_test_image, printed, restore, saved};

const CONFIG: &str = "/etc/pacman.conf";
const COPY: &str = "/etc/pacman.conf.pacnew";
const BACKUP: &str = "/var/lib/garuda-toolbox/backups/etc/pacman.conf";
const ADDED: &str = "# toolbox-test: a line the update brought\n";

fn with_pacnew(test: impl FnOnce(&str, &str) -> TestResult) -> TestResult {
    let before = saved(CONFIG)?;
    let current = fs::read_to_string(CONFIG)?;
    let new = format!("{current}{ADDED}");

    fs::write(COPY, &new)?;

    let result = test(&current, &new);

    restore(CONFIG, before.as_deref())?;
    restore(COPY, None)?;
    result
}

fn resolve(resolution: Resolution, current: &str, new: &str) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::ResolveMerge {
        merge: MergeResolve {
            copy: MergeCopy::new(COPY)?,
            choice: MergeChoice {
                resolution,
                copy: Fingerprint::of(new.as_bytes()),
                current: Some(Fingerprint::of(current.as_bytes())),
            },
        },
    })
}

#[test]
fn a_pacnew_is_handed_out_by_fetch() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    with_pacnew(|_, new| {
        let output = crate::oneshot::output(&["fetch", COPY])?;

        assert_eq!(output.status.code(), Some(0_i32));
        assert_eq!(
            FetchAnswer::parse(String::from_utf8(output.stdout)?.trim_end())?,
            FetchAnswer::Text {
                path: ConfigFilePath::new(COPY)?,
                content: ConfigText::new(new)?,
            }
        );
        Ok(())
    })
}

#[test]
fn taking_the_pacnew_replaces_the_file_and_keeps_a_backup() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    with_pacnew(|current, new| {
        assert_eq!(
            apply(vec![
                RootAction::SnapshotBeforeMerges,
                resolve(Resolution::TakeCopy, current, new)?,
            ])?,
            [Outcome::Skipped, Outcome::Succeeded]
        );
        assert_eq!(fs::read_to_string(CONFIG)?, new);
        assert_eq!(fs::read_to_string(BACKUP)?, current);
        assert!(!Path::new(COPY).exists());
        Ok(())
    })
}

#[test]
fn keeping_the_file_removes_only_the_pacnew() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    with_pacnew(|current, new| {
        assert_eq!(
            apply(vec![resolve(Resolution::KeepCurrent, current, new)?])?,
            [Outcome::Succeeded]
        );
        assert_eq!(fs::read_to_string(CONFIG)?, current);
        assert!(!Path::new(COPY).exists());
        Ok(())
    })
}

#[test]
fn a_pacnew_of_a_file_no_package_backs_up_stays() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let copy = "/etc/toolbox-test-unowned.pacnew";
    fs::write(copy, "x\n")?;

    let action = RootAction::ResolveMerge {
        merge: MergeResolve {
            copy: MergeCopy::new(copy)?,
            choice: MergeChoice {
                resolution: Resolution::KeepCurrent,
                copy: Fingerprint::of(b"x\n"),
                current: None,
            },
        },
    };

    let output = helper(&["apply"], &request(Mode::Run, vec![action])?)?;
    let kept = Path::new(copy).exists();

    restore(copy, None)?;
    assert!(printed(&output)?.contains("pacman lists no backup file"));
    assert!(kept);
    Ok(())
}
