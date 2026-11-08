mod cancel;

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use tempfile::TempDir;
use toolbox_exec::ConfigRead;
use toolbox_plan::{ConfigFilePath, ConfigText, Editor, FetchRefusal, HomePath};
use toolbox_state::configs::{ConfigEntryLocation, ConfigEntrySpec, Found, Link};

use super::{ConfigFileSource as _, HelperFetch, ReadProblem, SystemConfigFiles};
use crate::pkexec::Refusal;
use crate::testing::running_as_root;

/// Stands in for pkexec and the helper: a script that prints an answer.
struct Script(PathBuf);

impl HelperFetch for Script {
    fn fetch_command(&self, path: &ConfigFilePath) -> Command {
        let mut command = Command::new("/bin/sh");

        command.arg(&self.0).arg(path.as_str());
        command
    }
}

struct System {
    dir: TempDir,
}

impl System {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        let home = dir.path().join("home");

        fs::create_dir_all(root.join("etc/modprobe.d")).unwrap();
        fs::create_dir_all(root.join("usr/bin")).unwrap();
        fs::create_dir_all(root.join("usr/lib")).unwrap();
        fs::create_dir_all(home.join(".config/systemd/user")).unwrap();
        fs::write(root.join("etc/fstab"), "/dev/sda1 / btrfs\n").unwrap();
        fs::write(root.join("usr/lib/os-release"), "NAME=Garuda\n").unwrap();
        symlink("../usr/lib/os-release", root.join("etc/os-release")).unwrap();
        fs::write(root.join("etc/modprobe.d/nvidia.conf"), "").unwrap();
        fs::write(root.join("etc/modprobe.d/amdgpu.conf"), "").unwrap();
        symlink("nvidia.conf", root.join("etc/modprobe.d/link.conf")).unwrap();
        fs::write(home.join(".bashrc"), "alias ll='ls -l'\n").unwrap();
        fs::write(home.join(".config/systemd/user/sync.service"), "").unwrap();
        symlink(
            home.join(".bashrc"),
            home.join(".config/systemd/user/linked.service"),
        )
        .unwrap();
        fs::write(dir.path().join("helper.sh"), "exit 1\n").unwrap();
        Self { dir }
    }

    fn root(&self) -> PathBuf {
        self.dir.path().join("root")
    }

    fn helper(&self, script: &str) {
        fs::write(self.dir.path().join("helper.sh"), script).unwrap();
    }

    fn source(&self) -> SystemConfigFiles<Script> {
        SystemConfigFiles::new(
            self.root(),
            self.dir.path().join("home"),
            Script(self.dir.path().join("helper.sh")),
        )
    }

    /// A file the user may not read, as `/etc/sudoers` is.
    fn locked(&self) -> ConfigFilePath {
        let path = self.root().join("etc/locked.conf");

        fs::write(&path, "secret\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        ConfigFilePath::new("/etc/locked.conf").unwrap()
    }
}

fn spec(id: &str, location: ConfigEntryLocation, directory: bool) -> ConfigEntrySpec {
    ConfigEntrySpec {
        id: id.to_owned(),
        location,
        directory,
    }
}

fn root(id: &str, path: &str, directory: bool) -> ConfigEntrySpec {
    spec(
        id,
        ConfigEntryLocation::Root(ConfigFilePath::new(path).unwrap()),
        directory,
    )
}

fn home(id: &str, path: &str, directory: bool) -> ConfigEntrySpec {
    spec(
        id,
        ConfigEntryLocation::Home(HomePath::new(path).unwrap()),
        directory,
    )
}

fn specs() -> Vec<ConfigEntrySpec> {
    vec![
        root("fstab", "/etc/fstab", false),
        root("os-release", "/etc/os-release", false),
        root("missing", "/etc/missing.conf", false),
        root("modprobe", "/etc/modprobe.d", true),
        root("folder-as-file", "/etc/modprobe.d", false),
        home("bash", ".bashrc", false),
        home("units", ".config/systemd/user", true),
        home("zsh", ".zshrc", false),
    ]
}

#[test]
fn the_listing_holds_what_exists_of_the_catalog() {
    let system = System::new();
    let listing = system.source().listing(&specs());

    assert_eq!(listing.get("fstab"), Some(&Found::File { link: None }));
    assert_eq!(
        listing.get("os-release"),
        Some(&Found::File {
            link: Some(Link::refused("/usr/lib/os-release", FetchRefusal::Outside))
        })
    );
    assert_eq!(listing.get("missing"), None);
    assert_eq!(
        listing.get("modprobe"),
        Some(&Found::Folder(vec![
            "amdgpu.conf".to_owned(),
            "nvidia.conf".to_owned()
        ]))
    );
    assert_eq!(listing.get("folder-as-file"), None);
    assert_eq!(listing.get("bash"), Some(&Found::File { link: None }));
    assert_eq!(
        listing.get("units"),
        Some(&Found::Folder(vec![
            "linked.service".to_owned(),
            "sync.service".to_owned()
        ]))
    );
    assert_eq!(listing.get("zsh"), None);
    assert_eq!(listing.len(), 5);
}

#[test]
fn a_readable_root_file_is_read_without_the_helper() {
    let system = System::new();

    system.helper("echo 'the helper must not run' >&2\nexit 9\n");

    assert_eq!(
        system
            .source()
            .read(&ConfigFilePath::new("/etc/fstab").unwrap(), &Arc::default())
            .unwrap()
            .content
            .as_str(),
        "/dev/sda1 / btrfs\n"
    );
    assert_eq!(
        system.source().read(
            &ConfigFilePath::new("/etc/os-release").unwrap(),
            &Arc::default()
        ),
        Err(ReadProblem::Refused(FetchRefusal::Outside))
    );
    assert_eq!(
        system.source().read(
            &ConfigFilePath::new("/etc/missing.conf").unwrap(),
            &Arc::default()
        ),
        Err(ReadProblem::Refused(FetchRefusal::Missing))
    );
}

#[test]
fn a_file_kept_from_the_user_comes_from_the_helper() {
    if running_as_root() {
        return;
    }

    let system = System::new();
    let locked = system.locked();

    system.helper(
        "[ \"$1\" = /etc/locked.conf ] || exit 3\necho '{\"result\":\"text\",\"path\":\"/etc/real/locked.conf\",\"content\":\"secret\\n\"}'\n",
    );
    assert_eq!(
        system.source().read(&locked, &Arc::default()).unwrap(),
        ConfigRead {
            path: ConfigFilePath::new("/etc/real/locked.conf").unwrap(),
            content: ConfigText::new("secret\n").unwrap()
        }
    );

    system.helper("echo '{\"result\":\"refused\",\"reason\":\"not-text\"}'\n");
    assert_eq!(
        system.source().read(&locked, &Arc::default()),
        Err(ReadProblem::Refused(FetchRefusal::NotText))
    );
}

#[test]
fn a_dismissed_password_or_a_broken_answer_is_no_text() {
    if running_as_root() {
        return;
    }

    let system = System::new();
    let locked = system.locked();

    for (words, code, refusal) in [
        ("", 126_i32, Refusal::Dismissed),
        ("", 127_i32, Refusal::Denied),
        (
            "Error executing command as another user: No authentication agent found.",
            127_i32,
            Refusal::NoAgent,
        ),
        (
            "Cannot run program /usr/lib/garuda-toolbox/garuda-toolbox-helper: No such file or directory",
            127_i32,
            Refusal::HelperMissing,
        ),
    ] {
        system.helper(&format!("echo '{words}' >&2\nexit {code}\n"));
        assert_eq!(
            system.source().read(&locked, &Arc::default()),
            Err(ReadProblem::NotRun(refusal)),
            "{words}"
        );
    }

    system.helper("exit 1\n");
    assert!(matches!(
        system.source().read(&locked, &Arc::default()),
        Err(ReadProblem::Failed(reason)) if reason.contains("ended with")
    ));

    for answer in [
        "",
        "secret",
        "{\"result\":\"text\",\"path\":\"/etc/locked.conf\",\"content\":\"a\\u0000b\"}",
        "{\"result\":\"text\",\"content\":\"a\"}",
        "{\"result\":\"text\",\"path\":\"/nix/store/x\",\"content\":\"a\"}",
        "{\"result\":\"text\",\"path\":\"/etc/shadow\",\"content\":\"a\"}",
        "{\"result\":\"text\"}",
    ] {
        system.helper(&format!("echo '{answer}'\n"));
        assert!(
            matches!(
                system.source().read(&locked, &Arc::default()),
                Err(ReadProblem::Failed(reason)) if reason.contains("no config file")
            ),
            "{answer}"
        );
    }
}

#[test]
fn an_editor_is_installed_when_its_program_is_at_its_fixed_path() {
    let system = System::new();

    fs::write(system.root().join("usr/bin/nano"), "").unwrap();
    fs::create_dir_all(system.root().join("usr/bin/vim")).unwrap();

    assert!(system.source().installed(Editor::Nano));
    assert!(!system.source().installed(Editor::Micro));
    assert!(!system.source().installed(Editor::Vim));
}

#[test]
fn a_home_file_is_below_the_home() {
    let system = System::new();

    assert_eq!(
        system
            .source()
            .home_file(&HomePath::new(".config/kwinrc").unwrap()),
        Some(system.dir.path().join("home/.config/kwinrc"))
    );
}

/// `fstab` and `modprobe.d` moved into `etc/real` with links in their
/// place, and a link to nothing where `missing.conf` would be.
fn linked() -> System {
    let system = System::new();
    let etc = system.root().join("etc");

    fs::create_dir_all(etc.join("real")).unwrap();
    fs::rename(etc.join("fstab"), etc.join("real/fstab")).unwrap();
    symlink("real/fstab", etc.join("fstab")).unwrap();
    symlink("gone.conf", etc.join("missing.conf")).unwrap();
    fs::rename(etc.join("modprobe.d"), etc.join("real/modprobe.d")).unwrap();
    symlink("real/modprobe.d", etc.join("modprobe.d")).unwrap();
    system.helper("echo 'the helper must not run' >&2\nexit 9\n");
    system
}

#[test]
fn a_link_is_listed_with_where_it_leads() {
    let listing = linked().source().listing(&specs());

    assert_eq!(
        listing.get("fstab"),
        Some(&Found::File {
            link: Some(Link::to(&ConfigFilePath::new("/etc/real/fstab").unwrap()))
        })
    );
    assert_eq!(
        listing.get("missing"),
        Some(&Found::File {
            link: Some(Link::refused("/etc/gone.conf", FetchRefusal::Dangling))
        })
    );
    assert_eq!(
        listing.get("modprobe"),
        Some(&Found::Folder(vec![
            "amdgpu.conf".to_owned(),
            "nvidia.conf".to_owned()
        ]))
    );
    assert_eq!(
        listing.get("folder-as-file"),
        Some(&Found::File {
            link: Some(Link::refused(
                "/etc/real/modprobe.d",
                FetchRefusal::NotRegular
            ))
        })
    );
}

#[test]
fn a_link_is_read_as_the_file_it_leads_to() {
    let system = linked();
    let read = |path: &str| {
        system
            .source()
            .read(&ConfigFilePath::new(path).unwrap(), &Arc::default())
    };

    assert_eq!(
        read("/etc/fstab"),
        Ok(ConfigRead {
            path: ConfigFilePath::new("/etc/real/fstab").unwrap(),
            content: ConfigText::new("/dev/sda1 / btrfs\n").unwrap()
        })
    );
    assert_eq!(
        read("/etc/modprobe.d/nvidia.conf").unwrap().path.as_str(),
        "/etc/real/modprobe.d/nvidia.conf"
    );
    assert_eq!(
        read("/etc/missing.conf"),
        Err(ReadProblem::Refused(FetchRefusal::Dangling))
    );
}

#[test]
fn a_shared_helper_answers_for_any_owner() {
    let system = System::new();
    let helper: Arc<Script> = Arc::new(Script(system.dir.path().join("helper.sh")));
    let path = ConfigFilePath::new("/etc/fstab").unwrap();

    assert_eq!(
        HelperFetch::fetch_command(&helper, &path).get_program(),
        std::ffi::OsString::from("/bin/sh")
    );
}
