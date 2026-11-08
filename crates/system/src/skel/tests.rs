use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::TempDir;
use toolbox_catalog::{SkelCatalog, SkelEntry};

use super::{Backup, SkelError, available, reset};

/// Bash, GTK and ZSH, as `skel.toml` lists them.
const CATALOG: &str = r#"
[[config]]
id = "bash"
name = "Bash"
files = ["/etc/skel/.bashrc_garuda", "/etc/skel/.config/autostart/bashrc-setup.desktop"]

[[config]]
id = "gtk"
name = "GTK3/4"
files = ["/etc/skel/.config/gtk-3.0", "/etc/skel/.config/gtk-4.0"]

[[config]]
id = "zsh"
name = "ZSH"
files = ["/etc/skel/.zshrc"]
"#;

struct Roots {
    skel: TempDir,
    home: TempDir,
}

impl Roots {
    fn new() -> Self {
        Self {
            skel: tempfile::tempdir().unwrap(),
            home: tempfile::tempdir().unwrap(),
        }
    }

    fn skel(&self) -> &Path {
        self.skel.path()
    }

    fn home(&self) -> &Path {
        self.home.path()
    }
}

/// 2026-10-09 13:05:07 UTC.
fn now() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_791_551_107)
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn specs() -> Vec<SkelEntry> {
    SkelCatalog::parse(CATALOG, "skel.toml")
        .unwrap()
        .entries()
        .to_vec()
}

fn entry<'specs>(specs: &'specs [SkelEntry], id: &str) -> &'specs SkelEntry {
    specs.iter().find(|spec| spec.id() == id).unwrap()
}

#[test]
fn available_lists_entries_with_at_least_one_skel_file() {
    let roots = Roots::new();
    let specs = specs();

    write(
        &roots.skel().join(".config/autostart/bashrc-setup.desktop"),
        "x",
    );
    fs::create_dir_all(roots.skel().join(".config/gtk-4.0")).unwrap();

    let ids: Vec<&str> = available(&specs, roots.skel())
        .into_iter()
        .map(SkelEntry::id)
        .collect();

    assert_eq!(ids, ["bash", "gtk"]);
}

#[test]
fn reset_copies_files_into_new_folders() {
    let roots = Roots::new();
    let specs = specs();

    write(&roots.skel().join(".bashrc_garuda"), "garuda bash");
    write(
        &roots.skel().join(".config/autostart/bashrc-setup.desktop"),
        "desktop",
    );

    let report = reset(entry(&specs, "bash"), roots.skel(), roots.home(), now()).unwrap();

    assert_eq!(read(&roots.home().join(".bashrc_garuda")), "garuda bash");
    assert_eq!(
        read(&roots.home().join(".config/autostart/bashrc-setup.desktop")),
        "desktop"
    );
    assert_eq!(
        report.copied,
        [
            roots.home().join(".bashrc_garuda"),
            roots.home().join(".config/autostart/bashrc-setup.desktop")
        ]
    );
    assert_eq!(report.backups, Vec::new());
    assert_eq!(report.skipped, Vec::<PathBuf>::new());
}

#[test]
fn reset_copies_folders_recursively_with_their_links() {
    let roots = Roots::new();
    let specs = specs();
    let gtk = roots.skel().join(".config/gtk-3.0");

    write(&gtk.join("settings.ini"), "dark");
    write(&gtk.join("nested/deeper/a.css"), "css");
    symlink("settings.ini", gtk.join("link.ini")).unwrap();

    let report = reset(entry(&specs, "gtk"), roots.skel(), roots.home(), now()).unwrap();
    let copied = roots.home().join(".config/gtk-3.0");

    assert_eq!(read(&copied.join("settings.ini")), "dark");
    assert_eq!(read(&copied.join("nested/deeper/a.css")), "css");
    assert_eq!(
        fs::read_link(copied.join("link.ini")).unwrap(),
        Path::new("settings.ini")
    );
    assert_eq!(report.copied, [copied]);
    assert_eq!(report.skipped, [roots.skel().join(".config/gtk-4.0")]);
}

#[test]
fn reset_backs_up_existing_files_and_folders() {
    let roots = Roots::new();
    let specs = specs();
    let target = roots.home().join(".config/gtk-3.0");

    write(
        &roots.skel().join(".config/gtk-3.0/settings.ini"),
        "default",
    );
    write(&target.join("settings.ini"), "mine");
    write(&target.join("extra.css"), "extra");

    let report = reset(entry(&specs, "gtk"), roots.skel(), roots.home(), now()).unwrap();
    let backup = roots.home().join(".config/gtk-3.0.bak-20261009-130507");

    assert_eq!(read(&target.join("settings.ini")), "default");
    assert!(
        !target.join("extra.css").exists(),
        "the folder is replaced, not merged"
    );
    assert_eq!(read(&backup.join("settings.ini")), "mine");
    assert_eq!(read(&backup.join("extra.css")), "extra");
    assert_eq!(report.backups, [Backup { target, backup }]);
}

#[test]
fn a_second_reset_in_the_same_second_keeps_the_first_backup() {
    let roots = Roots::new();
    let specs = specs();
    let zsh = entry(&specs, "zsh");
    let target = roots.home().join(".zshrc");

    write(&roots.skel().join(".zshrc"), "default");
    write(&target, "mine");
    reset(zsh, roots.skel(), roots.home(), now()).unwrap();

    let report = reset(zsh, roots.skel(), roots.home(), now()).unwrap();
    let second = roots.home().join(".zshrc.bak-20261009-130507-1");

    assert_eq!(
        read(&roots.home().join(".zshrc.bak-20261009-130507")),
        "mine"
    );
    assert_eq!(read(&second), "default");
    assert_eq!(
        report.backups,
        [Backup {
            target,
            backup: second
        }]
    );
}

#[test]
fn a_symlink_target_is_backed_up_not_followed() {
    let roots = Roots::new();
    let specs = specs();
    let elsewhere = tempfile::tempdir().unwrap();
    let outside = elsewhere.path().join("zshrc");

    write(&outside, "outside");
    write(&roots.skel().join(".zshrc"), "default");
    symlink(&outside, roots.home().join(".zshrc")).unwrap();

    let report = reset(entry(&specs, "zsh"), roots.skel(), roots.home(), now()).unwrap();
    let backup = roots.home().join(".zshrc.bak-20261009-130507");

    assert_eq!(read(&outside), "outside");
    assert_eq!(read(&roots.home().join(".zshrc")), "default");
    assert_eq!(fs::read_link(&backup).unwrap(), outside);
    assert_eq!(report.backups.len(), 1);
}

#[test]
fn missing_skel_files_are_skipped() {
    let roots = Roots::new();
    let specs = specs();

    write(&roots.skel().join(".bashrc_garuda"), "garuda bash");

    let report = reset(entry(&specs, "bash"), roots.skel(), roots.home(), now()).unwrap();

    assert_eq!(report.copied, [roots.home().join(".bashrc_garuda")]);
    assert_eq!(
        report.skipped,
        [roots.skel().join(".config/autostart/bashrc-setup.desktop")]
    );
    assert!(!roots.home().join(".config").exists());
}

#[test]
fn a_folder_link_leaving_home_is_refused_before_anything_changes() {
    let roots = Roots::new();
    let specs = specs();
    let elsewhere = tempfile::tempdir().unwrap();

    write(&roots.skel().join(".bashrc_garuda"), "garuda bash");
    write(
        &roots.skel().join(".config/autostart/bashrc-setup.desktop"),
        "desktop",
    );
    write(&roots.home().join(".bashrc_garuda"), "mine");
    symlink(elsewhere.path(), roots.home().join(".config")).unwrap();

    let error = reset(entry(&specs, "bash"), roots.skel(), roots.home(), now()).unwrap_err();

    assert!(
        matches!(&error, SkelError::LinkLeavesHome { path } if *path == roots.home().join(".config")),
        "{error:?}"
    );
    assert_eq!(read(&roots.home().join(".bashrc_garuda")), "mine");
    assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 0);
}

#[test]
fn a_dangling_folder_link_is_refused() {
    let roots = Roots::new();
    let specs = specs();

    write(
        &roots.skel().join(".config/gtk-3.0/settings.ini"),
        "default",
    );
    symlink("/nonexistent/toolbox-test", roots.home().join(".config")).unwrap();

    let error = reset(entry(&specs, "gtk"), roots.skel(), roots.home(), now()).unwrap_err();

    assert!(
        matches!(error, SkelError::LinkLeavesHome { path: _ }),
        "{error:?}"
    );
}

#[test]
fn a_folder_link_inside_home_is_followed() {
    let roots = Roots::new();
    let specs = specs();
    let dotfiles = roots.home().join("dotfiles/config");

    fs::create_dir_all(&dotfiles).unwrap();
    symlink(&dotfiles, roots.home().join(".config")).unwrap();
    write(
        &roots.skel().join(".config/gtk-4.0/settings.ini"),
        "default",
    );

    reset(entry(&specs, "gtk"), roots.skel(), roots.home(), now()).unwrap();

    assert_eq!(read(&dotfiles.join("gtk-4.0/settings.ini")), "default");
}

#[test]
fn a_home_reached_through_a_link_works() {
    let roots = Roots::new();
    let specs = specs();
    let links = tempfile::tempdir().unwrap();
    let home = links.path().join("home");

    symlink(roots.home(), &home).unwrap();
    write(&roots.skel().join(".zshrc"), "default");

    let report = reset(entry(&specs, "zsh"), roots.skel(), &home, now()).unwrap();

    assert_eq!(read(&roots.home().join(".zshrc")), "default");
    assert_eq!(report.copied, [home.join(".zshrc")]);
}

#[test]
fn errors_name_the_path() {
    let roots = Roots::new();
    let specs = specs();
    let home = roots.home().join("missing");

    write(&roots.skel().join(".zshrc"), "default");

    let error = reset(entry(&specs, "zsh"), roots.skel(), &home, now()).unwrap_err();

    assert!(error.to_string().contains("missing"), "{error}");
}
