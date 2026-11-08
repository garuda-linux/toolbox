use std::fs;
use std::path::{Path, PathBuf};

use super::{AppIconSource as _, AppStreamIcons, file_path, file_url};

/// The first 300 files of `/usr/share/swcatalog/icons` in the Lima VM,
/// as `find . -maxdepth 3 -type f` lists them.
const SWCATALOG: &str = include_str!("fixtures/swcatalog-icons");
const EXTRA_128: &str = "archlinux-arch-extra/128x128";

fn touch(root: &Path, relative: &str) -> PathBuf {
    let path = root.join(relative);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "").unwrap();
    path
}

/// The captured files as empty files below `root`.
fn captured(root: &Path) {
    for file in SWCATALOG.lines() {
        touch(root, file.strip_prefix("./").unwrap());
    }
}

fn icon(root: &Path, name: &str) -> PathBuf {
    root.join(EXTRA_128).join(name)
}

#[test]
fn a_package_finds_its_cached_icon() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("swcatalog/icons");

    captured(&root);

    let icons = AppStreamIcons::scan(&[&root]);

    assert_eq!(icons.len(), 300);
    assert_eq!(
        icons.icon_file("firefox"),
        Some(icon(&root, "firefox_firefox.jxl"))
    );
    assert_eq!(
        icons.icon_file("calibre"),
        Some(icon(&root, "calibre_calibre-ebook-edit.jxl")),
        "the first of several icons"
    );
    assert_eq!(icons.icon_file("fire"), None, "a prefix is no package");
    assert_eq!(icons.icon_file(""), None);
}

/// The captured cache with a 64x64 copy of firefox's icon and a gimp icon
/// that only `core` has in 48x48.
#[test]
fn the_largest_size_wins() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    captured(root);
    touch(root, "archlinux-arch-extra/64x64/firefox_firefox.png");

    let small = touch(root, "archlinux-arch-core/48x48/gimp_gimp.png");
    let icons = AppStreamIcons::scan(&[root]);

    assert_eq!(icons.len(), 302);
    assert_eq!(
        icons.icon_file("firefox"),
        Some(icon(root, "firefox_firefox.jxl"))
    );
    assert_eq!(icons.icon_file("gimp"), Some(small));
}

/// The captured cache with a `digikam.svg` added.
#[test]
fn an_icon_named_like_the_package_wins() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    captured(root);

    assert_eq!(
        AppStreamIcons::scan(&[root]).icon_file("digikam"),
        Some(icon(root, "digikam_digikam.jxl"))
    );

    let plain = touch(root, &format!("{EXTRA_128}/digikam.svg"));

    assert_eq!(
        AppStreamIcons::scan(&[root]).icon_file("digikam"),
        Some(plain)
    );
}

/// The captured cache twice, in the current and in the old folder, with
/// made-up files the scan must skip.
#[test]
fn the_newer_root_wins_and_other_files_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("swcatalog");
    let old = dir.path().join("app-info");

    captured(&current);
    captured(&old);
    touch(&current, &format!("{EXTRA_128}/readme_notes.txt"));
    touch(&current, "archlinux-arch-extra/32x32/tiny_tiny.png");

    let icons = AppStreamIcons::scan(&[&current, &old, &dir.path().join("missing")]);

    assert_eq!(
        icons.icon_file("amarok"),
        Some(icon(&current, "amarok_amarok.jxl"))
    );
    assert_eq!(icons.icon_file("readme"), None);
    assert_eq!(icons.icon_file("tiny"), None, "only the listed sizes");
}

#[test]
fn without_a_cache_nothing_is_found() {
    let icons = AppStreamIcons::scan(&[Path::new("/nonexistent/swcatalog")]);

    assert!(icons.is_empty());
    assert_eq!(icons.icon_file("firefox"), None);
}

/// The made-up path needs three escapes. A captured one needs none.
#[test]
fn file_urls_encode_what_a_url_cannot_hold() {
    assert_eq!(
        file_url(Path::new("/usr/share/a b/c#d%e.png")),
        "file:///usr/share/a%20b/c%23d%25e.png"
    );
    assert_eq!(
        file_url(Path::new(
            "/usr/share/swcatalog/icons/archlinux-arch-extra/128x128/amberol_io.bassi.Amberol.jxl"
        )),
        "file:///usr/share/swcatalog/icons/archlinux-arch-extra/128x128/amberol_io.bassi.Amberol.jxl"
    );
}

/// Arch's cache holds JPEG XL files only. A text file beside them is
/// added.
#[test]
fn jpeg_xl_icons_are_found() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    captured(root);
    touch(root, &format!("{EXTRA_128}/notes.txt"));

    let icons = AppStreamIcons::scan(&[root]);

    assert_eq!(icons.len(), 300);
    assert!(SWCATALOG.lines().all(|file| {
        Path::new(file)
            .extension()
            .is_some_and(|extension| extension == "jxl")
    }));
}

#[test]
fn a_file_url_turns_back_into_its_path() {
    let path = "/usr/share/a b/c#d%e.png";

    assert_eq!(file_path(&file_url(Path::new(path))).as_deref(), Some(path));
    assert_eq!(
        file_path("file:///tmp/plain.png").as_deref(),
        Some("/tmp/plain.png")
    );
}

/// Hostile URLs, made up.
#[test]
fn only_local_well_formed_file_urls_have_a_path() {
    assert_eq!(file_path("https://example.org/a.png"), None);
    assert_eq!(file_path("file://host/a.png"), None);
    assert_eq!(file_path("file:///a%2.png"), None);
    assert_eq!(file_path("file:///a%FF.png"), None);
}
