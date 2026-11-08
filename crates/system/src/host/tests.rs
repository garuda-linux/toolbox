use std::ffi::OsString;
use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, SystemTime};

use tempfile::TempDir;
use toolbox_plan::UserName;

use super::{
    HostRoots, HostSource as _, SystemHost, parse_boot_time, parse_desktops, parse_edition,
    root_is_live,
};
use crate::ProbeError;

/// The live ISO session, whose root is the overlay `garuda-live`.
const OVERLAY: &str = include_str!("fixtures/live/proc-self-mounts");
/// The Lima VM, whose root is a btrfs subvolume.
const BTRFS: &str = include_str!("fixtures/proc-self-mounts");
const STAT: &str = include_str!("fixtures/proc-stat");
/// The `btime` of [`STAT`].
const BOOTED: u64 = 1_794_155_377;
const STATUS: &str = include_str!("fixtures/proc-self-status");
const RELEASE: &str = include_str!("fixtures/garuda-release");
const OS_RELEASE: &str = include_str!("fixtures/os-release");
/// The Lima VM's accounts: `user` is 1000, `guest` 1001, `nopass` 1002.
const PASSWD: &str = include_str!("../accounts/fixtures/passwd");

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> + Send + Sync + use<> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();

    move |key| {
        pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| OsString::from(value))
    }
}

fn system() -> (TempDir, HostRoots) {
    let root = tempfile::tempdir().unwrap();
    let roots = HostRoots {
        root: root.path().to_path_buf(),
        proc: root.path().join("proc"),
        garuda_lib: root.path().join("usr/lib/garuda"),
        garuda_state: root.path().join("var/lib/garuda"),
        etc: root.path().join("etc"),
    };

    for folder in [
        &roots.proc,
        &roots.garuda_lib,
        &roots.garuda_state,
        &roots.etc,
    ] {
        fs::create_dir_all(folder).unwrap();
    }

    (root, roots)
}

fn touch(path: &Path, seconds: u64) {
    File::create(path)
        .unwrap()
        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
        .unwrap();
}

/// No capture has an aufs root, so the live root line is renamed to one.
#[test]
fn an_overlay_or_aufs_root_is_a_live_system() {
    let aufs = OVERLAY.replacen("garuda-live / overlay", "none / aufs", 1);

    assert_eq!(root_is_live(OVERLAY), Some(true));
    assert_eq!(root_is_live(&aufs), Some(true));
    assert_eq!(root_is_live(BTRFS), Some(false));
}

/// The live root line put before the VM's table, as a root switched later
/// in the boot leaves it.
#[test]
fn the_last_root_mount_wins() {
    let live_root = OVERLAY.lines().next().unwrap();

    assert_eq!(root_is_live(&format!("{live_root}\n{BTRFS}")), Some(false));
}

/// The VM's table without its root line, with an empty and a broken line
/// appended.
#[test]
fn mounts_without_a_root_say_nothing() {
    let (_root_line, rest) = BTRFS.split_once('\n').unwrap();

    assert_eq!(root_is_live(&format!("{rest}\ngarbage\n")), None);
}

/// The installed release with its edition quoted, indented and in capitals,
/// as older releases wrote it.
#[test]
fn the_edition_is_read_without_quotes_in_lower_case() {
    let quoted = RELEASE.replacen("EDITION=mokka", "  EDITION=\"Dr460nized\"", 1);
    let single = RELEASE.replacen("EDITION=mokka", "EDITION='KDE-lite'", 1);

    assert_eq!(parse_edition(RELEASE), Some("mokka".to_owned()));
    assert_eq!(parse_edition(&quoted), Some("dr460nized".to_owned()));
    assert_eq!(parse_edition(&single), Some("kde-lite".to_owned()));
}

/// `os-release` names no edition. The installed release is edited to an
/// empty one.
#[test]
fn a_release_without_an_edition_has_none() {
    let empty = RELEASE.replacen("EDITION=mokka", "EDITION=\"\"", 1);

    assert_eq!(parse_edition(OS_RELEASE), None);
    assert_eq!(parse_edition(&empty), None);
    assert_eq!(parse_edition(""), None);
}

/// The VM's `/proc/stat`, then with its `btime` line dropped and with a
/// value that is no number.
#[test]
fn the_boot_time_comes_from_btime() {
    let without: String = STAT
        .lines()
        .filter(|line| !line.starts_with("btime "))
        .flat_map(|line| [line, "\n"])
        .collect();

    let odd = STAT.replacen(&format!("btime {BOOTED}"), "btime soon", 1);

    assert_eq!(parse_boot_time(STAT), Some(BOOTED));
    assert_eq!(parse_boot_time(&without), None);
    assert_eq!(parse_boot_time(&odd), None);
}

#[test]
fn desktops_are_colon_separated() {
    assert_eq!(parse_desktops("KDE"), vec!["KDE".to_owned()]);
    assert_eq!(
        parse_desktops("ubuntu:GNOME"),
        vec!["ubuntu".to_owned(), "GNOME".to_owned()]
    );
    assert_eq!(parse_desktops(""), Vec::<String>::new());
}

/// The VM's `/proc/self/status` with its `Uid:` line set to `real` and
/// `effective`.
fn running_as(roots: &HostRoots, real: u32, effective: u32) {
    fs::create_dir_all(roots.proc.join("self")).unwrap();
    fs::write(
        roots.proc.join("self/status"),
        STATUS.replacen(
            "Uid:\t1000\t1000\t1000\t1000",
            &format!("Uid:\t{real}\t{effective}\t{effective}\t{effective}"),
            1,
        ),
    )
    .unwrap();
}

#[test]
fn the_user_is_the_effective_user_id_not_the_environment() {
    let (_root, roots) = system();

    fs::write(roots.etc.join("passwd"), PASSWD).unwrap();
    running_as(&roots, 1000, 1001);

    // `sudo -u guest` or `su guest` without `-l` keeps `USER` of user.
    let host = SystemHost::new(roots, env(&[("USER", "user"), ("LOGNAME", "user")]));

    assert_eq!(host.user(), Ok(UserName::new("guest").unwrap()));
}

#[test]
fn the_unchanged_status_names_the_vms_user() {
    let (_root, roots) = system();

    fs::write(roots.etc.join("passwd"), PASSWD).unwrap();
    fs::create_dir_all(roots.proc.join("self")).unwrap();
    fs::write(roots.proc.join("self/status"), STATUS).unwrap();

    let host = SystemHost::new(roots, env(&[]));

    assert_eq!(host.user(), Ok(UserName::new("user").unwrap()));
}

#[test]
fn a_user_id_missing_from_passwd_names_no_user() {
    let (_root, roots) = system();

    fs::write(roots.etc.join("passwd"), PASSWD).unwrap();
    running_as(&roots, 1003, 1003);

    let host = SystemHost::new(roots, env(&[("USER", "user")]));

    assert_eq!(host.user(), Err(ProbeError::Unavailable));
}

#[test]
fn the_desktop_comes_from_the_environment() {
    let (_root, roots) = system();

    let plasma = SystemHost::new(roots.clone(), env(&[("XDG_CURRENT_DESKTOP", "KDE")]));
    let console = SystemHost::new(roots, env(&[]));

    assert_eq!(plasma.desktops(), Ok(vec!["KDE".to_owned()]));
    assert_eq!(console.desktops(), Ok(Vec::new()));
}

#[test]
fn live_iso_reads_the_mount_table() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots.clone(), env(&[]));

    assert_eq!(host.is_live_iso(), Err(ProbeError::Unavailable));

    fs::create_dir_all(roots.proc.join("self")).unwrap();
    fs::write(roots.proc.join("self/mounts"), OVERLAY).unwrap();
    assert_eq!(host.is_live_iso(), Ok(true));

    fs::write(roots.proc.join("self/mounts"), BTRFS).unwrap();
    assert_eq!(host.is_live_iso(), Ok(false));

    fs::write(
        roots.proc.join("self/mounts"),
        include_str!("fixtures/installed/proc-self-mounts"),
    )
    .unwrap();
    assert_eq!(host.is_live_iso(), Ok(false));

    fs::write(roots.proc.join("self/mounts"), "").unwrap();
    assert!(matches!(host.is_live_iso(), Err(ProbeError::Failed(_))));
}

/// Docker, podman and distrobox run on an overlay root too.
#[test]
fn a_container_is_no_live_iso() {
    let (_root, roots) = system();
    fs::create_dir_all(roots.proc.join("self")).unwrap();
    fs::write(roots.proc.join("self/mounts"), OVERLAY).unwrap();

    let podman = SystemHost::new(roots.clone(), env(&[("container", "podman")]));
    assert_eq!(podman.is_live_iso(), Ok(false));

    let host = SystemHost::new(roots.clone(), env(&[]));
    fs::write(roots.root.join(".dockerenv"), "").unwrap();
    assert_eq!(host.is_live_iso(), Ok(false));

    fs::remove_file(roots.root.join(".dockerenv")).unwrap();
    fs::create_dir_all(roots.root.join("run")).unwrap();
    fs::write(roots.root.join("run/.containerenv"), "").unwrap();
    assert_eq!(host.is_live_iso(), Ok(false));
}

#[test]
fn the_edition_is_optional() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots.clone(), env(&[]));

    assert_eq!(host.edition(), Ok(None));

    fs::write(roots.garuda_lib.join("garuda-release"), RELEASE).unwrap();
    assert_eq!(host.edition(), Ok(Some("mokka".to_owned())));

    fs::write(
        roots.garuda_lib.join("garuda-release"),
        include_str!("fixtures/live/garuda-release"),
    )
    .unwrap();
    assert_eq!(host.edition(), Ok(Some("mokka".to_owned())));
}

#[test]
fn an_update_after_boot_needs_a_reboot() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots.clone(), env(&[]));
    fs::write(roots.proc.join("stat"), STAT).unwrap();

    touch(&roots.garuda_state.join("last_update"), BOOTED + 1000);
    assert_eq!(host.is_reboot_pending(), Ok(true));

    touch(&roots.garuda_state.join("last_update"), BOOTED - 1000);
    assert_eq!(host.is_reboot_pending(), Ok(false));
}

#[test]
fn no_update_stamp_means_no_reboot() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots, env(&[]));

    assert_eq!(host.is_reboot_pending(), Ok(false));
}

/// The VM's `/proc/stat` cut before its `btime` line.
#[test]
fn an_update_stamp_without_boot_time_is_not_judged() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots.clone(), env(&[]));
    touch(&roots.garuda_state.join("last_update"), BOOTED + 1000);

    assert_eq!(host.is_reboot_pending(), Err(ProbeError::Unavailable));

    let (before, _btime) = STAT.split_once("btime ").unwrap();
    fs::write(roots.proc.join("stat"), before).unwrap();
    assert!(matches!(
        host.is_reboot_pending(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn the_process_host_reads_the_real_system() {
    let host = SystemHost::from_process();

    assert_ne!(host.is_live_iso(), Err(ProbeError::Unavailable));
    host.edition().unwrap();
    host.is_reboot_pending().unwrap();
}
