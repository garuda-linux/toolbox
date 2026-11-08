use std::fs;

use tempfile::TempDir;
use toolbox_state::network::{Category, ModuleFacts};

use super::{ModuleSource as _, SystemModules};
use crate::ProbeError;
use crate::testing::write;

const RELEASE: &str = "6.17.1-zen1-1-zen";

fn system() -> (TempDir, SystemModules) {
    let root = tempfile::tempdir().unwrap();

    write(
        root.path(),
        "proc/sys/kernel/osrelease",
        format!("{RELEASE}\n"),
    );
    write(
        root.path(),
        "proc/modules",
        "iwlwifi 602112 1 iwlmvm, Live 0x0\nkvm 1404928 0 - Live 0x0\n",
    );
    write(
        root.path(),
        &format!("usr/lib/modules/{RELEASE}/modules.dep"),
        "kernel/drivers/net/wireless/intel/iwlwifi/iwlwifi.ko.zst:\n\
         kernel/drivers/net/ethernet/realtek/r8169.ko.zst:\n\
         kernel/arch/x86/kvm/kvm.ko.zst:\n",
    );

    let modules = SystemModules::new(root.path());

    (root, modules)
}

fn blacklists(facts: &ModuleFacts) -> Vec<(String, String, bool)> {
    facts
        .blacklists
        .iter()
        .map(|entry| (entry.module.to_string(), entry.file.clone(), entry.own))
        .collect()
}

#[test]
fn loaded_and_loadable_modules_come_from_proc_and_modules_dep() {
    let (_root, modules) = system();
    let facts = modules.facts().unwrap();

    let loaded: Vec<String> = facts.loaded.iter().map(ToString::to_string).collect();
    let network: Vec<(String, Category)> = facts
        .modules
        .iter()
        .map(|module| (module.name.to_string(), module.category))
        .collect();

    assert_eq!(loaded, ["iwlwifi", "kvm"]);
    assert_eq!(
        network,
        [
            ("iwlwifi".to_owned(), Category::Wireless),
            ("r8169".to_owned(), Category::Ethernet),
        ]
    );
    assert_eq!(facts.blacklists, []);
}

#[test]
fn every_conf_file_of_the_modprobe_folders_is_read_and_named() {
    let (root, modules) = system();

    write(
        root.path(),
        "etc/modprobe.d/garuda-toolbox-blacklist.conf",
        "# Written by Garuda Toolbox\nblacklist r8169\n",
    );
    write(
        root.path(),
        "etc/modprobe.d/blacklist.conf",
        "blacklist pcspkr\n# blacklist iwlwifi\ninstall b43 /bin/false\n",
    );
    write(
        root.path(),
        "usr/lib/modprobe.d/broadcom-wl.conf",
        "blacklist b43\nblacklist brcm-smac\n",
    );
    write(
        root.path(),
        "run/modprobe.d/live.conf",
        "blacklist nouveau\n",
    );
    write(
        root.path(),
        "etc/modprobe.d/notes.txt",
        "blacklist iwlwifi\n",
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

#[test]
fn a_file_is_read_once_from_the_first_folder_that_has_its_name() {
    let (root, modules) = system();

    for (file, text) in [
        ("usr/lib/modprobe.d/a.conf", "blacklist from_usr_a\n"),
        ("run/modprobe.d/a.conf", "blacklist from_run_a\n"),
        ("etc/modprobe.d/a.conf", ""),
        ("usr/lib/modprobe.d/b.conf", "blacklist from_usr_b\n"),
        ("run/modprobe.d/b.conf", "blacklist from_run_b\n"),
        ("usr/lib/modprobe.d/c.conf", "blacklist from_usr_c\n"),
    ] {
        write(root.path(), file, text);
    }

    assert_eq!(
        blacklists(&modules.facts().unwrap()),
        [
            (
                "from_run_b".to_owned(),
                "/run/modprobe.d/b.conf".to_owned(),
                false
            ),
            (
                "from_usr_c".to_owned(),
                "/usr/lib/modprobe.d/c.conf".to_owned(),
                false
            ),
        ]
    );
}

/// A comment in Latin-1 must not hide the blacklist below it.
#[test]
fn a_stray_byte_in_a_conf_file_hides_no_blacklist() {
    let (root, modules) = system();

    write(
        root.path(),
        "etc/modprobe.d/old.conf",
        b"# caf\xe9\nblacklist pcspkr\n",
    );

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

    fs::remove_dir_all(root.path().join("usr/lib/modules").join(RELEASE)).unwrap();

    let facts = modules.facts().unwrap();

    assert_eq!(facts.modules, []);
    assert_eq!(facts.loaded.len(), 2);
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
