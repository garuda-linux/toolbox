use std::fs;
use std::path::{Path, PathBuf};

use super::{AppIconSource as _, AppStreamIcons, file_path, file_url};

fn touch(root: &Path, relative: &str) -> PathBuf {
    let path = root.join(relative);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "").unwrap();
    path
}

#[test]
fn a_package_finds_its_cached_icon_in_the_largest_size() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("swcatalog/icons");
    let large = touch(&root, "archlinux-arch-extra/128x128/firefox_firefox.png");

    touch(&root, "archlinux-arch-extra/64x64/firefox_firefox.png");

    let small = touch(&root, "archlinux-arch-core/48x48/gimp_gimp.png");
    let icons = AppStreamIcons::scan(&[&root]);

    assert_eq!(icons.len(), 2);
    assert_eq!(icons.icon_file("firefox"), Some(large));
    assert_eq!(icons.icon_file("gimp"), Some(small));
    assert_eq!(icons.icon_file("fire"), None, "a prefix is no package");
    assert_eq!(icons.icon_file(""), None);
}

#[test]
fn an_icon_named_like_the_package_wins() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    touch(root, "repo/64x64/kate_org.kde.kate.png");

    let plain = touch(root, "repo/64x64/kate.svg");

    assert_eq!(AppStreamIcons::scan(&[root]).icon_file("kate"), Some(plain));
}

#[test]
fn the_newer_root_wins_and_other_files_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("swcatalog");
    let old = dir.path().join("app-info");
    let new_icon = touch(&current, "arch/64x64/vlc_vlc.png");

    touch(&old, "arch/64x64/vlc_vlc.png");
    touch(&current, "arch/64x64/readme_notes.txt");
    touch(&current, "arch/32x32/tiny_tiny.png");

    let icons = AppStreamIcons::scan(&[&current, &old, &dir.path().join("missing")]);

    assert_eq!(icons.icon_file("vlc"), Some(new_icon));
    assert_eq!(icons.icon_file("readme"), None);
    assert_eq!(icons.icon_file("tiny"), None, "only the listed sizes");
}

#[test]
fn without_a_cache_nothing_is_found() {
    let icons = AppStreamIcons::scan(&[Path::new("/nonexistent/swcatalog")]);

    assert!(icons.is_empty());
    assert_eq!(icons.icon_file("firefox"), None);
}

#[test]
fn file_urls_encode_what_a_url_cannot_hold() {
    assert_eq!(
        file_url(Path::new("/usr/share/a b/c#d%e.png")),
        "file:///usr/share/a%20b/c%23d%25e.png"
    );
    assert_eq!(
        file_url(Path::new("/usr/share/swcatalog/icons/x_y-1.0~2.png")),
        "file:///usr/share/swcatalog/icons/x_y-1.0~2.png"
    );
}

/// Arch's cache holds JPEG XL files (`firefox_firefox.jxl`).
#[test]
fn jpeg_xl_icons_are_found() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let icon = touch(root, "archlinux-arch-extra/128x128/firefox_firefox.jxl");

    touch(root, "archlinux-arch-extra/128x128/notes.txt");

    let icons = AppStreamIcons::scan(&[root]);

    assert_eq!(icons.len(), 1);
    assert_eq!(icons.icon_file("firefox"), Some(icon));
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

#[test]
fn only_local_well_formed_file_urls_have_a_path() {
    assert_eq!(file_path("https://example.org/a.png"), None);
    assert_eq!(file_path("file://host/a.png"), None);
    assert_eq!(file_path("file:///a%2.png"), None);
    assert_eq!(file_path("file:///a%FF.png"), None);
}
