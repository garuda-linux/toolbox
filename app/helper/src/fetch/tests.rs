use std::fs;
use std::os::unix::fs::symlink;

use toolbox_plan::{ConfigFilePath, ConfigText, FetchAnswer, FetchRefusal, MAX_CONFIG_BYTES};

use super::answer;

fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

fn refusal(reason: FetchRefusal) -> FetchAnswer {
    FetchAnswer::Refused { reason }
}

fn fetch(root: &std::path::Path, text: &str) -> FetchAnswer {
    let line = answer(root, &path(text)).unwrap();

    assert_eq!(line.matches('\n').count(), 1, "{line}");
    FetchAnswer::parse(line.trim_end()).unwrap()
}

#[test]
fn a_config_file_is_answered_as_one_line_of_text() {
    let root = tempfile::tempdir().unwrap();

    fs::create_dir_all(root.path().join("etc/default")).unwrap();
    fs::write(
        root.path().join("etc/default/grub"),
        "GRUB_TIMEOUT=5\nGRUB_CMDLINE_LINUX=\"quiet\"\n",
    )
    .unwrap();

    assert_eq!(
        answer(root.path(), &path("/etc/default/grub")).unwrap(),
        "{\"result\":\"text\",\"path\":\"/etc/default/grub\",\"content\":\"GRUB_TIMEOUT=5\\nGRUB_CMDLINE_LINUX=\\\"quiet\\\"\\n\"}\n"
    );
}

#[test]
fn a_link_is_answered_with_the_text_and_the_path_of_its_target() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");

    fs::create_dir_all(etc.join("real")).unwrap();
    fs::write(etc.join("real/app.conf"), "a = 1\n").unwrap();
    symlink("real/app.conf", etc.join("app.conf")).unwrap();
    symlink("/etc/app.conf", etc.join("chain.conf")).unwrap();

    for link in ["/etc/app.conf", "/etc/chain.conf"] {
        assert_eq!(
            answer(root.path(), &path(link)).unwrap(),
            "{\"result\":\"text\",\"path\":\"/etc/real/app.conf\",\"content\":\"a = 1\\n\"}\n",
            "{link}"
        );
    }
}

#[test]
fn a_link_that_leads_to_no_config_file_is_refused_with_its_reason() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");

    for folder in ["etc/folder", "usr/lib", "run", "nix/store", "root"] {
        fs::create_dir_all(root.path().join(folder)).unwrap();
    }

    for (file, content) in [
        ("usr/lib/os-release", "NAME=x\n"),
        ("run/resolv.conf", "nameserver ::1\n"),
        ("nix/store/abc-fstab", "/ / ext4\n"),
        ("root/secret", "key\n"),
        ("etc/shadow", "root:hash\n"),
    ] {
        fs::write(root.path().join(file), content).unwrap();
    }

    symlink("../usr/lib/os-release", etc.join("os-release")).unwrap();
    symlink("/run/resolv.conf", etc.join("resolv.conf")).unwrap();
    symlink("/nix/store/abc-fstab", etc.join("fstab")).unwrap();
    symlink("../root/secret", etc.join("secret")).unwrap();
    symlink("shadow", etc.join("hashes")).unwrap();
    symlink("/", etc.join("up")).unwrap();
    symlink("gone", etc.join("dangling")).unwrap();
    symlink("loop", etc.join("loop")).unwrap();
    symlink("folder", etc.join("folder-link")).unwrap();

    for link in [
        "/etc/os-release",
        "/etc/resolv.conf",
        "/etc/fstab",
        "/etc/secret",
        "/etc/hashes",
        "/etc/up/root/secret",
    ] {
        assert_eq!(
            fetch(root.path(), link),
            refusal(FetchRefusal::Outside),
            "{link}"
        );
    }

    for link in ["/etc/dangling", "/etc/loop"] {
        assert_eq!(
            fetch(root.path(), link),
            refusal(FetchRefusal::Dangling),
            "{link}"
        );
    }

    assert_eq!(
        fetch(root.path(), "/etc/folder-link"),
        refusal(FetchRefusal::NotRegular)
    );
}

#[test]
fn what_is_no_plain_text_file_is_refused_with_its_reason() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");

    fs::create_dir_all(etc.join("folder")).unwrap();
    fs::write(etc.join("binary"), [0x7f, b'E', b'L', b'F', 0x00]).unwrap();
    fs::write(etc.join("big"), vec![b'x'; MAX_CONFIG_BYTES + 1]).unwrap();

    assert_eq!(
        fetch(root.path(), "/etc/missing"),
        refusal(FetchRefusal::Missing)
    );
    assert_eq!(
        fetch(root.path(), "/etc/folder"),
        refusal(FetchRefusal::NotRegular)
    );
    assert_eq!(
        fetch(root.path(), "/etc/binary"),
        refusal(FetchRefusal::NotText)
    );
    assert_eq!(
        fetch(root.path(), "/etc/big"),
        refusal(FetchRefusal::TooLarge)
    );
}

#[test]
fn the_largest_file_still_fits_the_answer() {
    let root = tempfile::tempdir().unwrap();
    let content = "\u{1}".repeat(MAX_CONFIG_BYTES);

    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/full"), &content).unwrap();

    assert_eq!(
        fetch(root.path(), "/etc/full"),
        FetchAnswer::Text {
            path: path("/etc/full"),
            content: ConfigText::new(content).unwrap()
        }
    );
}

#[test]
fn an_unreadable_file_is_an_error_not_an_answer() {
    if rustix::process::geteuid().is_root() {
        return;
    }

    let root = tempfile::tempdir().unwrap();

    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/locked"), "x\n").unwrap();
    fs::set_permissions(
        root.path().join("etc/locked"),
        std::os::unix::fs::PermissionsExt::from_mode(0o000),
    )
    .unwrap();

    assert_eq!(
        answer(root.path(), &path("/etc/locked"))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
}
