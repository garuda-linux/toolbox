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

const OVERLAY: &str = "proc /proc proc rw,nosuid 0 0\n\
                       airootfs / overlay rw,relatime,lowerdir=/run/archiso 0 0\n\
                       tmpfs /tmp tmpfs rw 0 0\n";

const BTRFS: &str = "/dev/nvme0n1p2 / btrfs rw,noatime,subvol=/@ 0 0\n\
                     /dev/nvme0n1p2 /home btrfs rw,noatime,subvol=/@home 0 0\n";

const STAT: &str = "cpu  10 20 30 40\n\
                    intr 12345\n\
                    btime 1000\n\
                    processes 77\n";

const PASSWD: &str = "root:x:0:0::/root:/bin/bash\n\
                      nico:x:1000:1000:Nico:/home/nico:/bin/fish\n\
                      ada:x:1001:1001::/home/ada:/bin/bash\n";

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

#[test]
fn an_overlay_or_aufs_root_is_a_live_system() {
    assert_eq!(root_is_live(OVERLAY), Some(true));
    assert_eq!(root_is_live("none / aufs rw 0 0\n"), Some(true));
    assert_eq!(root_is_live(BTRFS), Some(false));
}

#[test]
fn the_last_root_mount_wins() {
    let mounts = "rootfs / rootfs rw 0 0\n/dev/sda2 / ext4 rw 0 0\n";

    assert_eq!(root_is_live(mounts), Some(false));
}

#[test]
fn mounts_without_a_root_say_nothing() {
    assert_eq!(root_is_live("tmpfs /tmp tmpfs rw 0 0\n\ngarbage\n"), None);
}

#[test]
fn the_edition_is_read_without_quotes_in_lower_case() {
    let release = "VERSION=\"240101\"\n  EDITION=\"Dr460nized\"\nCODENAME=x\n";

    assert_eq!(parse_edition(release), Some("dr460nized".to_owned()));
    assert_eq!(parse_edition("EDITION=gnome\n"), Some("gnome".to_owned()));
    assert_eq!(
        parse_edition("EDITION='KDE-lite'\n"),
        Some("kde-lite".to_owned())
    );
}

#[test]
fn a_release_without_an_edition_has_none() {
    assert_eq!(parse_edition("VERSION=1\n"), None);
    assert_eq!(parse_edition("EDITION=\"\"\n"), None);
    assert_eq!(parse_edition(""), None);
}

#[test]
fn the_boot_time_comes_from_btime() {
    assert_eq!(parse_boot_time(STAT), Some(1000));
    assert_eq!(parse_boot_time("cpu 1 2\n"), None);
    assert_eq!(parse_boot_time("btime soon\n"), None);
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

fn running_as(roots: &HostRoots, real: u32, effective: u32) {
    fs::create_dir_all(roots.proc.join("self")).unwrap();
    fs::write(
        roots.proc.join("self/status"),
        format!("Name:\tgaruda-toolbox\nUid:\t{real}\t{effective}\t{effective}\t{effective}\n"),
    )
    .unwrap();
}

#[test]
fn the_user_is_the_effective_user_id_not_the_environment() {
    let (_root, roots) = system();

    fs::write(roots.etc.join("passwd"), PASSWD).unwrap();
    running_as(&roots, 1000, 1001);

    // `sudo -u ada` or `su ada` without `-l` keeps `USER` of nico.
    let host = SystemHost::new(roots, env(&[("USER", "nico"), ("LOGNAME", "nico")]));

    assert_eq!(host.user(), Ok(UserName::new("ada").unwrap()));
}

#[test]
fn a_user_id_missing_from_passwd_names_no_user() {
    let (_root, roots) = system();

    fs::write(roots.etc.join("passwd"), PASSWD).unwrap();
    running_as(&roots, 1002, 1002);

    let host = SystemHost::new(roots, env(&[("USER", "nico")]));

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

    fs::write(roots.garuda_lib.join("garuda-release"), "EDITION=Mokka\n").unwrap();
    assert_eq!(host.edition(), Ok(Some("mokka".to_owned())));
}

#[test]
fn an_update_after_boot_needs_a_reboot() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots.clone(), env(&[]));
    fs::write(roots.proc.join("stat"), STAT).unwrap();

    touch(&roots.garuda_state.join("last_update"), 2000);
    assert_eq!(host.is_reboot_pending(), Ok(true));

    touch(&roots.garuda_state.join("last_update"), 500);
    assert_eq!(host.is_reboot_pending(), Ok(false));
}

#[test]
fn no_update_stamp_means_no_reboot() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots, env(&[]));

    assert_eq!(host.is_reboot_pending(), Ok(false));
}

#[test]
fn an_update_stamp_without_boot_time_is_not_judged() {
    let (_root, roots) = system();
    let host = SystemHost::new(roots.clone(), env(&[]));
    touch(&roots.garuda_state.join("last_update"), 2000);

    assert_eq!(host.is_reboot_pending(), Err(ProbeError::Unavailable));

    fs::write(roots.proc.join("stat"), "cpu 1 2 3\n").unwrap();
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
