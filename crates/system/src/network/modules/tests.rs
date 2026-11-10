use std::fs;

use tempfile::TempDir;
use toolbox_plan::{KernelModule, set_blacklisted};
use toolbox_state::network::{Category, ModuleFacts};

use super::{ModuleSource as _, SystemModules};
use crate::ProbeError;
use crate::test_support::write;

const RELEASE: &str = include_str!("fixtures/osrelease");
const PROC_MODULES: &str = include_str!("fixtures/proc-modules");
const MODULES_DEP: &str = include_str!("fixtures/modules.dep");
/// The Lima VM with the Broadcom `wl` driver built by DKMS and loaded, next
/// to a USB Wi-Fi stick.
const PROC_MODULES_WIFI: &str = include_str!("fixtures/proc-modules-wifi");
const MODULES_DEP_DKMS: &str = crate::testing::network::modules::MODULES_DEP_DKMS;
/// The only modprobe files of the Lima VM, from its `usr/lib/modprobe.d`.
const SYSTEMD_CONF: &str =
    toolbox_plan::testing::network::modules::ROOT_USR_LIB_MODPROBE_D_SYSTEMD_CONF;
const README: &str = include_str!("fixtures/root/usr/lib/modprobe.d/README");

fn system_with(proc_modules: &str, modules_dep: &str) -> (TempDir, SystemModules) {
    let root = tempfile::tempdir().unwrap();

    write(root.path(), "proc/sys/kernel/osrelease", RELEASE);
    write(root.path(), "proc/modules", proc_modules);
    write(
        root.path(),
        &format!("usr/lib/modules/{}/modules.dep", RELEASE.trim()),
        modules_dep,
    );
    write(root.path(), "usr/lib/modprobe.d/systemd.conf", SYSTEMD_CONF);
    write(root.path(), "usr/lib/modprobe.d/README", README);

    let modules = SystemModules::new(root.path());

    (root, modules)
}

fn system() -> (TempDir, SystemModules) {
    system_with(PROC_MODULES, MODULES_DEP)
}

/// The VM's `systemd.conf` with `lines` appended: the VM blacklists
/// nothing, so every blacklist line of these tests is such an edit.
fn conf_with(lines: &str) -> String {
    format!("{SYSTEMD_CONF}{lines}")
}

fn blacklists(facts: &ModuleFacts) -> Vec<(String, String, bool)> {
    facts
        .blacklists
        .iter()
        .map(|entry| (entry.module.to_string(), entry.file.clone(), entry.own))
        .collect()
}

fn category(facts: &ModuleFacts, name: &str) -> Option<Category> {
    facts
        .modules
        .iter()
        .find(|module| module.name.as_str() == name)
        .map(|module| module.category)
}

#[test]
fn loaded_and_loadable_modules_come_from_proc_and_modules_dep() {
    let (_root, modules) = system();
    let facts = modules.facts().unwrap();
    let loaded: Vec<String> = facts.loaded.iter().map(ToString::to_string).collect();

    assert_eq!(loaded.len(), PROC_MODULES.lines().count());
    assert!(loaded.iter().any(|name| name == "virtio_net"), "{loaded:?}");
    assert_eq!(category(&facts, "r8169"), Some(Category::Ethernet));
    assert_eq!(category(&facts, "iwlwifi"), Some(Category::Wireless));
    assert_eq!(category(&facts, "mt7921e"), Some(Category::Wireless));
    assert_eq!(category(&facts, "virtio_net"), Some(Category::Other));
    assert_eq!(category(&facts, "btusb"), None);
    assert_eq!(facts.blacklists, []);
}

/// DKMS puts `wl` below `updates/`, outside `kernel/drivers/net`.
#[test]
fn a_dkms_driver_is_a_loaded_wireless_module() {
    let (_root, modules) = system_with(PROC_MODULES_WIFI, MODULES_DEP_DKMS);
    let facts = modules.facts().unwrap();

    assert!(facts.loaded.iter().any(|name| name.as_str() == "wl"));
    assert!(facts.loaded.iter().any(|name| name.as_str() == "mt76x2u"));
    assert_eq!(category(&facts, "wl"), Some(Category::Wireless));
    assert_eq!(category(&facts, "v4l2loopback"), None);
}

#[test]
fn every_conf_file_of_the_modprobe_folders_is_read_and_named() {
    let (root, modules) = system();
    let own = set_blacklisted("", &KernelModule::new("r8169").unwrap(), true);

    write(
        root.path(),
        "etc/modprobe.d/garuda-toolbox-blacklist.conf",
        own,
    );
    write(
        root.path(),
        "etc/modprobe.d/blacklist.conf",
        conf_with("blacklist pcspkr\n# blacklist iwlwifi\ninstall b43 /bin/false\n"),
    );
    write(
        root.path(),
        "usr/lib/modprobe.d/broadcom-wl.conf",
        conf_with("blacklist b43\nblacklist brcm-smac\n"),
    );
    write(
        root.path(),
        "run/modprobe.d/live.conf",
        conf_with("blacklist nouveau\n"),
    );
    write(
        root.path(),
        "etc/modprobe.d/notes.txt",
        conf_with("blacklist iwlwifi\n"),
    );

    assert_eq!(
        blacklists(&modules.facts().unwrap()),
        [
            (
                "pcspkr".to_owned(),
                "/etc/modprobe.d/blacklist.conf".to_owned(),
                false
            ),
            (
                "r8169".to_owned(),
                "/etc/modprobe.d/garuda-toolbox-blacklist.conf".to_owned(),
                true
            ),
            (
                "nouveau".to_owned(),
                "/run/modprobe.d/live.conf".to_owned(),
                false
            ),
            (
                "b43".to_owned(),
                "/usr/lib/modprobe.d/broadcom-wl.conf".to_owned(),
                false
            ),
            (
                "brcm_smac".to_owned(),
                "/usr/lib/modprobe.d/broadcom-wl.conf".to_owned(),
                false
            ),
        ]
    );
}

/// The VM's `systemd.conf` shadowed in `run` and `etc`, the way an
/// administrator turns a file of a package off.
#[test]
fn a_file_is_read_once_from_the_first_folder_that_has_its_name() {
    let (root, modules) = system();

    write(
        root.path(),
        "usr/lib/modprobe.d/systemd.conf",
        conf_with("blacklist from_usr\n"),
    );
    write(
        root.path(),
        "run/modprobe.d/systemd.conf",
        conf_with("blacklist from_run\n"),
    );
    write(
        root.path(),
        "usr/lib/modprobe.d/b.conf",
        conf_with("blacklist from_usr_b\n"),
    );
    write(
        root.path(),
        "run/modprobe.d/b.conf",
        conf_with("blacklist from_run_b\n"),
    );

    assert_eq!(
        blacklists(&modules.facts().unwrap()),
        [
            (
                "from_run_b".to_owned(),
                "/run/modprobe.d/b.conf".to_owned(),
                false
            ),
            (
                "from_run".to_owned(),
                "/run/modprobe.d/systemd.conf".to_owned(),
                false
            ),
        ]
    );

    write(root.path(), "etc/modprobe.d/systemd.conf", "");

    assert_eq!(
        blacklists(&modules.facts().unwrap()),
        [(
            "from_run_b".to_owned(),
            "/run/modprobe.d/b.conf".to_owned(),
            false
        )]
    );
}

/// A comment in Latin-1 must not hide the blacklist below it. The test
/// puts one such comment and a blacklist line after the VM's
/// `systemd.conf`.
#[test]
fn a_stray_byte_in_a_conf_file_hides_no_blacklist() {
    let (root, modules) = system();
    let mut text = SYSTEMD_CONF.as_bytes().to_vec();

    text.extend_from_slice(b"# caf\xe9\nblacklist pcspkr\n");
    write(root.path(), "etc/modprobe.d/old.conf", text);

    assert_eq!(
        blacklists(&modules.facts().unwrap()),
        [(
            "pcspkr".to_owned(),
            "/etc/modprobe.d/old.conf".to_owned(),
            false
        )]
    );
}

/// After a kernel update the running kernel's folder is gone until the
/// restart.
#[test]
fn a_running_kernel_without_its_folder_has_no_loadable_modules() {
    let (root, modules) = system();

    fs::remove_dir_all(root.path().join("usr/lib/modules").join(RELEASE.trim())).unwrap();

    let facts = modules.facts().unwrap();

    assert_eq!(facts.modules, []);
    assert_eq!(facts.loaded.len(), PROC_MODULES.lines().count());
}

#[test]
fn a_release_that_is_no_folder_name_reads_no_file() {
    let (root, modules) = system();

    write(root.path(), "proc/sys/kernel/osrelease", "../../etc\n");

    assert_eq!(modules.facts().unwrap().modules, []);
}

#[test]
fn without_proc_the_modules_are_unavailable() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        SystemModules::new(root.path()).facts(),
        Err(ProbeError::Unavailable)
    );
}
