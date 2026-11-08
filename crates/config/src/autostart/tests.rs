use std::fs;

use super::Autostart;

fn setup() -> (tempfile::TempDir, Autostart) {
    let dir = tempfile::tempdir().unwrap();
    let desktop_file = dir.path().join("share/garuda-toolbox.desktop");
    fs::create_dir_all(desktop_file.parent().unwrap()).unwrap();
    fs::write(&desktop_file, "[Desktop Entry]\n").unwrap();
    let autostart = Autostart::new(dir.path().join("config/autostart"), desktop_file);
    (dir, autostart)
}

#[test]
fn starts_disabled_without_a_link() {
    let (_dir, autostart) = setup();

    assert!(!autostart.is_enabled());
}

#[test]
fn enabling_links_the_installed_desktop_file() {
    let (dir, autostart) = setup();

    autostart.enable().unwrap();

    assert!(autostart.is_enabled());
    let link = dir.path().join("config/autostart/garuda-toolbox.desktop");
    assert_eq!(
        fs::read_link(link).unwrap(),
        dir.path().join("share/garuda-toolbox.desktop")
    );
}

#[test]
fn enabling_twice_replaces_the_old_link() {
    let (_dir, autostart) = setup();

    autostart.enable().unwrap();
    autostart.enable().unwrap();

    assert!(autostart.is_enabled());
}

#[test]
fn disabling_removes_the_link_and_the_old_rani_entries() {
    let (dir, autostart) = setup();
    let folder = dir.path().join("config/autostart");
    autostart.enable().unwrap();
    fs::write(folder.join("garuda-rani.desktop"), "").unwrap();
    fs::write(folder.join("org.garudalinux.rani.desktop"), "").unwrap();
    fs::write(folder.join("other.desktop"), "").unwrap();

    autostart.disable().unwrap();

    assert!(!autostart.is_enabled());
    assert!(!folder.join("garuda-rani.desktop").exists());
    assert!(!folder.join("org.garudalinux.rani.desktop").exists());
    assert!(folder.join("other.desktop").exists());
}

#[test]
fn enabling_removes_the_old_rani_entries() {
    let (dir, autostart) = setup();
    let folder = dir.path().join("config/autostart");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("garuda-rani.desktop"), "").unwrap();
    std::os::unix::fs::symlink("/nowhere", folder.join("org.garudalinux.rani.desktop")).unwrap();

    autostart.enable().unwrap();

    assert!(autostart.is_enabled());
    for name in ["garuda-rani.desktop", "org.garudalinux.rani.desktop"] {
        folder.join(name).symlink_metadata().unwrap_err();
    }
}

#[test]
fn disabling_without_a_link_is_fine() {
    let (_dir, autostart) = setup();

    autostart.disable().unwrap();
}

#[test]
fn a_dangling_link_still_counts_as_enabled() {
    let (dir, autostart) = setup();
    autostart.enable().unwrap();
    fs::remove_file(dir.path().join("share/garuda-toolbox.desktop")).unwrap();

    assert!(autostart.is_enabled());
    assert!(
        !dir.path()
            .join("config/autostart/garuda-toolbox.desktop")
            .exists()
    );
}
