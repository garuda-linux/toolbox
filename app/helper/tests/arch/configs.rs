use std::error::Error;
use std::fs;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _, chown, symlink};
use std::path::Path;
use std::process::Command;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{
    ConfigFilePath, ConfigText, ConfigWrite, FetchAnswer, FetchRefusal, Fingerprint, RootAction,
};

use super::common::{TestResult, helper, request};
use super::{apply, in_test_image, printed};

const FOLDER: &str = "/etc/systemd/network";
const FILE: &str = "/etc/systemd/network/toolbox-test-secret";
const BACKUP: &str = "/var/lib/garuda-toolbox/backups/etc/systemd/network/toolbox-test-secret";
const LINK_BACKUP: &str = "/var/lib/garuda-toolbox/backups/etc/systemd/network/toolbox-test-link";
const GONE: &str = "/etc/systemd/network/toolbox-test-gone";
const OLD: &str = "token = old\n";
const NEW: &str = "token = new\n# edited\n";

/// A file only root may read and run, owned by root and the test group.
/// It lies in a folder of the catalog because the helper writes nowhere
/// else. Only the test's own `toolbox-test-*` files there are removed.
fn fresh() -> TestResult {
    fs::create_dir_all(FOLDER)?;

    for entry in fs::read_dir(FOLDER)? {
        let entry = entry?;

        if entry
            .file_name()
            .to_string_lossy()
            .starts_with("toolbox-test-")
        {
            fs::remove_file(entry.path())?;
        }
    }

    fs::remove_file(BACKUP).or_else(ignore_missing)?;
    fs::write(FILE, OLD)?;
    fs::set_permissions(FILE, fs::Permissions::from_mode(0o700))?;

    let group = test_group()?;

    chown(FILE, Some(0), Some(group))?;
    Ok(())
}

/// The id of the group `ci/arch/setup.sh` creates.
fn test_group() -> Result<u32, Box<dyn Error>> {
    let groups = fs::read_to_string("/etc/group")?;
    let id = groups
        .lines()
        .find_map(|line| line.strip_prefix("toolbox-test-group:"))
        .and_then(|rest| rest.split(':').nth(1))
        .ok_or("the image has no toolbox-test-group")?;

    Ok(id.parse()?)
}

fn ignore_missing(error: std::io::Error) -> std::io::Result<()> {
    if error.kind() == std::io::ErrorKind::NotFound {
        Ok(())
    } else {
        Err(error)
    }
}

fn fetch(path: &str) -> Result<(Option<i32>, String), Box<dyn Error>> {
    let output = crate::oneshot::output(&["fetch", path])?;

    Ok((output.status.code(), String::from_utf8(output.stdout)?))
}

fn write(path: &str, content: &str, original: &str) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::WriteConfigFile {
        write: ConfigWrite {
            path: ConfigFilePath::new(path)?,
            content: ConfigText::new(content)?,
            original: Fingerprint::of(original.as_bytes()),
        },
    })
}

#[test]
fn a_file_only_root_reads_is_handed_out_as_text() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh()?;

    let unreadable = Command::new("su")
        .args(["tester", "-c", &format!("cat {FILE}")])
        .output()?;

    assert!(!unreadable.status.success(), "the test user can read it");

    let (code, answer) = fetch(FILE)?;

    assert_eq!(code, Some(0_i32));
    assert_eq!(
        FetchAnswer::parse(answer.trim_end())?,
        FetchAnswer::Text {
            path: ConfigFilePath::new(FILE)?,
            content: ConfigText::new(OLD)?
        }
    );
    Ok(())
}

#[test]
fn an_edit_is_written_back_with_owner_mode_and_a_backup() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh()?;

    let before = fs::metadata(FILE)?;

    assert_eq!(apply(vec![write(FILE, NEW, OLD)?])?, [Outcome::Succeeded]);

    let after = fs::metadata(FILE)?;
    let backup = fs::metadata(BACKUP)?;

    assert_eq!(fs::read_to_string(FILE)?, NEW);
    assert_eq!(after.permissions().mode() & 0o7777, 0o700);
    assert_eq!((after.uid(), after.gid()), (before.uid(), before.gid()));
    assert_ne!(after.gid(), 0, "the group was not kept");
    assert_eq!(fs::read_to_string(BACKUP)?, OLD);
    assert_eq!(backup.permissions().mode() & 0o7777, 0o600);
    assert_eq!((backup.uid(), backup.gid()), (0, 0));
    Ok(())
}

#[test]
fn a_path_outside_the_config_folders_is_refused() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    for path in [
        "/etc/shadow",
        "/root/.bashrc",
        "/etc/../root/.bashrc",
        "/usr/bin/pacman",
    ] {
        let (code, answer) = fetch(path)?;

        assert_eq!(code, Some(2_i32), "{path}");
        assert!(answer.is_empty(), "{path}");
    }

    let shadow = fs::read_to_string("/etc/shadow")?;
    let line = request(Mode::Run, vec![write("/etc/fstab", "x\n", "")?])?
        .replace("/etc/fstab", "/etc/shadow");

    let output = helper(&["apply"], &line)?;

    assert!(!output.status.success());
    assert!(output.stdout.contains("\"rejected\""), "{}", output.stdout);
    assert_eq!(fs::read_to_string("/etc/shadow")?, shadow);

    // Readable, but no file of the catalog: the request never runs.
    for path in ["/etc/ld.so.preload", "/etc/sudoers.d/toolbox-test"] {
        let unlisted =
            request(Mode::Run, vec![write("/etc/fstab", "x\n", "")?])?.replace("/etc/fstab", path);

        let refused = helper(&["apply"], &unlisted)?;

        assert!(!refused.status.success(), "{path}");
        assert!(
            refused.stdout.contains("never written"),
            "{}",
            refused.stdout
        );
        assert!(!Path::new(path).exists(), "{path}");
    }

    Ok(())
}

/// A link is edited as the file it leads to: the helper resolves it,
/// writes the target in the target's own folder and leaves the link.
#[test]
fn a_link_is_edited_as_its_target_and_stays_a_link() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh()?;

    let link = format!("{FOLDER}/toolbox-test-link");
    let chain = format!("{FOLDER}/toolbox-test-chain");

    symlink("toolbox-test-secret", &link)?;
    symlink(&link, &chain)?;

    let before = fs::metadata(FILE)?;
    let (code, answer) = fetch(&chain)?;

    assert_eq!(code, Some(0_i32));
    assert_eq!(
        FetchAnswer::parse(answer.trim_end())?,
        FetchAnswer::Text {
            path: ConfigFilePath::new(FILE)?,
            content: ConfigText::new(OLD)?
        }
    );
    assert_eq!(apply(vec![write(&chain, NEW, OLD)?])?, [Outcome::Succeeded]);

    let after = fs::metadata(FILE)?;

    assert_eq!(fs::read_to_string(FILE)?, NEW);
    assert_eq!(fs::read_link(&link)?, Path::new("toolbox-test-secret"));
    assert_eq!(fs::read_link(&chain)?, Path::new(&link));
    assert!(!fs::symlink_metadata(FILE)?.file_type().is_symlink());
    assert_eq!(after.permissions().mode() & 0o7777, 0o700);
    assert_eq!((after.uid(), after.gid()), (before.uid(), before.gid()));
    assert_eq!(fs::read_to_string(BACKUP)?, OLD);
    assert!(!Path::new(LINK_BACKUP).exists(), "the link got a backup");
    Ok(())
}

/// Makes the link `name` to `target` in the test folder and checks that
/// the helper neither hands out nor writes what it leads to.
fn refused_link(name: &str, target: &str, reason: FetchRefusal, words: &str) -> TestResult {
    let link = format!("{FOLDER}/toolbox-test-{name}");
    let kept = fs::read(target).ok();

    symlink(target, &link)?;

    let (code, answer) = fetch(&link)?;

    assert_eq!(code, Some(0_i32), "{name}");
    assert_eq!(
        FetchAnswer::parse(answer.trim_end())?,
        FetchAnswer::Refused { reason },
        "{name}"
    );

    let output = helper(
        &["apply"],
        &request(Mode::Run, vec![write(&link, NEW, OLD)?])?,
    )?;

    assert!(!output.status.success(), "{name}");
    assert!(
        printed(&output)?.contains(words),
        "{name}: {}",
        output.stdout
    );
    assert_eq!(fs::read(target).ok(), kept, "{name}");
    assert!(
        fs::symlink_metadata(&link)?.file_type().is_symlink(),
        "{name}"
    );
    Ok(())
}

#[test]
fn a_link_out_of_the_config_folders_is_neither_read_nor_written() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh()?;

    let victim = "/root/toolbox-test-victim";
    let outside = "outside the folders";

    fs::write(victim, OLD)?;
    refused_link("home.conf", victim, FetchRefusal::Outside, outside)?;
    refused_link(
        "package.conf",
        "/usr/lib/os-release",
        FetchRefusal::Outside,
        outside,
    )?;
    refused_link(
        "kernel.conf",
        "/proc/sys/kernel/hostname",
        FetchRefusal::Outside,
        outside,
    )?;
    refused_link("hashes.conf", "/etc/shadow", FetchRefusal::Outside, outside)?;
    refused_link(
        "dangling.conf",
        GONE,
        FetchRefusal::Dangling,
        "leads to no file",
    )?;
    refused_link(
        "folder.conf",
        "/etc/pacman.d",
        FetchRefusal::NotRegular,
        "not a regular file",
    )?;
    assert!(!Path::new(GONE).exists(), "a dangling link made a file");
    fs::remove_file(victim)?;
    Ok(())
}

#[test]
fn a_file_that_changed_meanwhile_is_kept() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    fresh()?;
    fs::write(FILE, "token = from someone else\n")?;

    let output = helper(
        &["apply"],
        &request(Mode::Run, vec![write(FILE, NEW, OLD)?])?,
    )?;

    assert!(!output.status.success());
    assert!(
        printed(&output)?.contains("changed since it was opened"),
        "{}",
        output.stdout
    );
    assert_eq!(fs::read_to_string(FILE)?, "token = from someone else\n");
    assert!(!Path::new(BACKUP).exists(), "a backup without a write");
    Ok(())
}

/// The page only shows the read-only files, and a run as root refuses to
/// write one too: the request never runs, so no file changes and no
/// backup appears.
#[test]
fn a_read_only_file_is_never_written() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    for path in ["/etc/sudoers", "/etc/passwd", "/etc/group"] {
        let kept = fs::read_to_string(path)?;
        let refused = helper(
            &["apply"],
            &request(Mode::Run, vec![write(path, "x\n", &kept)?])?,
        )?;

        assert!(!refused.status.success(), "{path}");
        assert!(
            refused.stdout.contains("never written"),
            "{}",
            refused.stdout
        );
        assert_eq!(fs::read_to_string(path)?, kept, "{path}");
        assert!(
            !Path::new("/var/lib/garuda-toolbox/backups")
                .join(path.trim_start_matches('/'))
                .exists(),
            "{path} got a backup"
        );
    }

    Ok(())
}
