use std::fs;
use std::path::Path;

use super::rebuild;

fn install(root: &Path, path: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, "").unwrap();
}

fn picked(root: &Path) -> Option<String> {
    rebuild(root).map(|command| command.to_string())
}

#[test]
fn garudas_script_comes_first_where_both_are_installed() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "usr/bin/kernel-install");
    install(root.path(), "usr/lib/garuda/kernel-install-uki");

    assert_eq!(
        picked(root.path()).as_deref(),
        Some("kernel-install-uki add")
    );
}

#[test]
fn kernel_install_builds_them_on_plain_arch() {
    let root = tempfile::tempdir().unwrap();
    install(root.path(), "usr/bin/kernel-install");

    assert_eq!(
        picked(root.path()).as_deref(),
        Some("kernel-install add-all")
    );
}

#[test]
fn nothing_is_picked_without_either() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("usr/bin/kernel-install")).unwrap();

    assert_eq!(picked(root.path()), None);
}
