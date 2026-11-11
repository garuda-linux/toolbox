use std::cell::Cell;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::{IconNames, Indexed};
use crate::desktop::icon_files::IconLookup;

/// Answers every name, and counts the asking.
#[derive(Default)]
struct Walk {
    asked: Cell<usize>,
}

impl IconLookup for Walk {
    fn find(&self, _theme: &str, name: &str) -> Option<PathBuf> {
        self.asked.set(self.asked.get() + 1);
        Some(PathBuf::from(format!("/found/{name}")))
    }
}

fn touch(root: &Path, file: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "").unwrap();
}

/// A theme as themes lay out their files, with a folder linked in from
/// another theme, a link back to its own top and a stray text file.
fn icons() -> TempDir {
    let root = TempDir::new().unwrap();

    touch(root.path(), "icons/breeze/apps/48/firefox.svg");
    touch(root.path(), "icons/hicolor/48x48/apps/org.kde.kate.png");
    touch(root.path(), "icons/hicolor/scalable/apps/README.txt");
    touch(root.path(), "pixmaps/htop.xpm");
    touch(root.path(), "elsewhere/apps/vlc.svg");
    symlink(
        root.path().join("elsewhere/apps"),
        root.path().join("icons/breeze/linked"),
    )
    .unwrap();
    symlink(
        root.path().join("icons"),
        root.path().join("icons/breeze/loop"),
    )
    .unwrap();
    root
}

fn names(root: &TempDir) -> IconNames {
    IconNames::read(&[root.path().join("icons"), root.path().join("pixmaps")])
}

#[test]
fn every_picture_under_the_folders_is_named_by_its_stem() {
    let root = icons();
    let names = names(&root);

    for name in ["firefox", "org.kde.kate", "htop", "vlc"] {
        assert!(names.has(name), "{name}");
    }

    assert!(!names.has("README"), "a text file is no picture");
    assert!(!names.has("firefox.svg"));
}

#[test]
fn a_name_no_folder_has_misses_without_the_walk() {
    let known = names(&icons());
    let lookup = Indexed::new(Walk::default(), move || known.clone());

    assert_eq!(lookup.find("breeze", "linux-headers"), None);
    assert_eq!(lookup.inner.asked.get(), 0);
}

#[test]
fn a_name_a_folder_has_goes_to_the_lookup_for_its_size_and_theme() {
    let known = names(&icons());
    let lookup = Indexed::new(Walk::default(), move || known.clone());

    assert_eq!(
        lookup.find("breeze", "firefox"),
        Some(PathBuf::from("/found/firefox"))
    );
    assert_eq!(lookup.inner.asked.get(), 1);
}

/// `freedesktop-icons` reads a name with a slash as a file's path.
#[test]
fn a_name_with_a_slash_goes_to_the_lookup() {
    let lookup = Indexed::new(Walk::default(), IconNames::default);

    assert!(lookup.find("breeze", "/opt/app/icon").is_some());
}
