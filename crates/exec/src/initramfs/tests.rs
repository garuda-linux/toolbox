use std::fs;
use std::path::Path;

use toolbox_plan::REPAIR_ROOT;
use toolbox_plan::procedure::InitramfsRoot;

use super::rebuild;

fn install(root: &Path, program: &str) {
    let bin = root.join("usr/bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join(program), "").unwrap();
}

fn picked(root: &Path) -> Option<String> {
    rebuild(root, InitramfsRoot::System).map(|command| command.to_string())
}

#[test]
fn dracut_rebuild_comes_first_where_both_are_installed() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "mkinitcpio");
    install(root.path(), "dracut-rebuild");

    assert_eq!(picked(root.path()).as_deref(), Some("dracut-rebuild"));
}

#[test]
fn mkinitcpio_builds_them_where_dracut_rebuild_is_missing() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "mkinitcpio");

    assert_eq!(picked(root.path()).as_deref(), Some("mkinitcpio -P"));
}

#[test]
fn nothing_is_picked_without_either() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("usr/bin/dracut-rebuild")).unwrap();

    assert_eq!(picked(root.path()), None, "a folder is no program");
}

/// The installed system decides, not the live ISO, and its program runs
/// inside it.
#[test]
fn a_repair_picks_the_installed_systems_program_and_runs_it_there() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "dracut-rebuild");
    install(&root.path().join(REPAIR_ROOT), "mkinitcpio");

    let command = rebuild(root.path(), InitramfsRoot::Repair).map(|command| command.to_string());

    assert_eq!(
        command.as_deref(),
        Some("chroot /run/garuda-toolbox/boot-repair/root /usr/bin/mkinitcpio -P")
    );
}
