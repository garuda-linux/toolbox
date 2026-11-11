use std::cell::Cell;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::{IconNames, Indexed};
use crate::desktop::icon_files::IconLookup;

/// `find /usr/share/icons/hicolor /usr/share/pixmaps -printf '%y %p %l\n'`
/// on the Garuda Lima image: type, path, link target.
const TREE: &str = include_str!("fixtures/installed/icons-tree");

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

/// The captured tree under a temporary root, its files empty.
fn rebuilt() -> TempDir {
    let root = TempDir::new().unwrap();

    for line in TREE.lines() {
        let mut parts = line.splitn(3, ' ');
        let (kind, path) = (parts.next().unwrap(), parts.next().unwrap());
        let at = root.path().join(path.trim_start_matches('/'));

        match kind {
            "d" => fs::create_dir_all(&at).unwrap(),
            "f" => fs::write(&at, "").unwrap(),
            "l" => symlink(parts.next().unwrap().trim_end(), &at).unwrap(),
            other => panic!("no such entry type {other}"),
        }
    }

    root
}

fn folders(root: &TempDir) -> [PathBuf; 2] {
    [
        root.path().join("usr/share/icons"),
        root.path().join("usr/share/pixmaps"),
    ]
}

fn names(root: &TempDir) -> IconNames {
    IconNames::read(&folders(root))
}

#[test]
fn every_picture_of_the_image_is_named_by_its_stem() {
    let names = names(&rebuilt());

    for name in [
        "btop",
        "gvim",
        "micro",
        "garuda-toolbox",
        "archlinux-logo",
        "pinentry",
    ] {
        assert!(names.has(name), "{name}");
    }

    assert!(!names.has("index"), "index.theme is no picture");
    assert!(!names.has("btop.svg"));
}

/// The capture has no links. Changed: a folder of pictures from outside
/// linked in, and a link from a theme back to the icons folder.
#[test]
fn a_linked_folder_is_read_once_and_a_link_back_up_ends() {
    let root = rebuilt();
    let elsewhere = root.path().join("opt/app/icons");
    fs::create_dir_all(&elsewhere).unwrap();
    fs::write(elsewhere.join("vlc.svg"), "").unwrap();

    let hicolor = root.path().join("usr/share/icons/hicolor");
    symlink(&elsewhere, hicolor.join("linked")).unwrap();
    symlink(root.path().join("usr/share/icons"), hicolor.join("loop")).unwrap();

    let names = names(&root);

    assert!(names.has("vlc"));
    assert!(names.has("btop"));
}

#[test]
fn a_name_no_folder_has_misses_without_the_walk() {
    let known = names(&rebuilt());
    let lookup = Indexed::new(Walk::default(), move || known.clone());

    assert_eq!(lookup.find("hicolor", "linux-headers"), None);
    assert_eq!(lookup.inner.asked.get(), 0);
}

#[test]
fn a_name_a_folder_has_goes_to_the_lookup_for_its_size_and_theme() {
    let known = names(&rebuilt());
    let lookup = Indexed::new(Walk::default(), move || known.clone());

    assert_eq!(
        lookup.find("hicolor", "btop"),
        Some(PathBuf::from("/found/btop"))
    );
    assert_eq!(lookup.inner.asked.get(), 1);
}

/// `freedesktop-icons` reads a name with a slash as a file's path.
#[test]
fn a_name_with_a_slash_goes_to_the_lookup() {
    let lookup = Indexed::new(Walk::default(), IconNames::default);

    assert!(lookup.find("hicolor", "/opt/app/icon").is_some());
}

#[test]
fn the_rebuilt_tree_is_the_captured_one() {
    let root = rebuilt();
    let files = TREE.lines().filter(|line| line.starts_with("f ")).count();
    let found = walk_files(&root.path().join("usr/share"));

    assert_eq!(found, files);
}

fn walk_files(folder: &Path) -> usize {
    fs::read_dir(folder)
        .unwrap()
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() { walk_files(&path) } else { 1 }
        })
        .sum()
}
