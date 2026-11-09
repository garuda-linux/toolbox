use std::fs;
use std::os::unix::fs::symlink;

use toolbox_plan::{ConfigFilePath, ConfigText, FetchAnswer, FetchRefusal, MAX_CONFIG_BYTES};

use super::answer;
use crate::testing::SHADOW;

/// `/etc/default/grub` of a Garuda install in QEMU.
const DEFAULT_GRUB: &str =
    include_str!("../../../../crates/plan/src/grub/fixtures/installed/default-grub");
/// The Lima VM's `/etc/os-release`.
const OS_RELEASE: &str = include_str!("../../../../crates/system/src/host/fixtures/os-release");

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
    fs::write(root.path().join("etc/default/grub"), DEFAULT_GRUB).unwrap();

    let line = answer(root.path(), &path("/etc/default/grub")).unwrap();

    assert!(
        line.starts_with(
            "{\"result\":\"text\",\"path\":\"/etc/default/grub\",\"content\":\"# GRUB boot loader"
        ),
        "{line}"
    );
    assert_eq!(
        fetch(root.path(), "/etc/default/grub"),
        FetchAnswer::Text {
            path: path("/etc/default/grub"),
            content: ConfigText::new(DEFAULT_GRUB.to_owned()).unwrap()
        }
    );
}

#[test]
fn a_link_is_answered_with_the_text_and_the_path_of_its_target() {
    let root = tempfile::tempdir().unwrap();
    let modprobe = root.path().join("etc/modprobe.d");

    fs::create_dir_all(&modprobe).unwrap();
    fs::write(modprobe.join("real.conf"), "a = 1\n").unwrap();
    symlink("real.conf", modprobe.join("app.conf")).unwrap();
    symlink("/etc/modprobe.d/app.conf", modprobe.join("chain.conf")).unwrap();

    for link in ["/etc/modprobe.d/app.conf", "/etc/modprobe.d/chain.conf"] {
        assert_eq!(
            answer(root.path(), &path(link)).unwrap(),
            "{\"result\":\"text\",\"path\":\"/etc/modprobe.d/real.conf\",\"content\":\"a = \
             1\\n\"}\n",
            "{link}"
        );
    }
}

/// The asked path is a listed one, but where its link leads decides: a
/// secret beside the listed files must not come out through a link.
#[test]
fn a_link_to_a_file_the_catalog_does_not_name_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");

    for folder in ["modprobe.d", "openvpn/client", "systemd/network"] {
        fs::create_dir_all(etc.join(folder)).unwrap();
    }

    fs::write(etc.join("openvpn/client/vpn.conf"), "<key>\nsecret\n").unwrap();
    fs::write(etc.join("systemd/network/wg0.netdev"), "PrivateKey=x\n").unwrap();
    fs::write(etc.join("sudoers"), "root ALL=(ALL) ALL\n").unwrap();
    symlink(
        "../openvpn/client/vpn.conf",
        etc.join("modprobe.d/vpn.conf"),
    )
    .unwrap();
    symlink(
        "../systemd/network/wg0.netdev",
        etc.join("modprobe.d/wg.conf"),
    )
    .unwrap();
    symlink("../sudoers", etc.join("modprobe.d/sudo.conf")).unwrap();

    for link in ["/etc/modprobe.d/vpn.conf", "/etc/modprobe.d/wg.conf"] {
        assert_eq!(
            fetch(root.path(), link),
            refusal(FetchRefusal::Outside),
            "{link}"
        );
    }

    assert_eq!(
        fetch(root.path(), "/etc/modprobe.d/sudo.conf"),
        FetchAnswer::Text {
            path: path("/etc/sudoers"),
            content: ConfigText::new("root ALL=(ALL) ALL\n").unwrap()
        }
    );
}

#[test]
fn a_link_that_leads_to_no_config_file_is_refused_with_its_reason() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc");

    for folder in ["etc/folder", "usr/lib", "run", "nix/store", "root"] {
        fs::create_dir_all(root.path().join(folder)).unwrap();
    }

    for (file, content) in [
        ("usr/lib/os-release", OS_RELEASE),
        ("run/resolv.conf", "nameserver ::1\n"),
        ("nix/store/abc-fstab", "/ / ext4\n"),
        ("root/secret", "key\n"),
        ("etc/shadow", SHADOW),
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

    fs::create_dir_all(root.path().join("etc/modprobe.d")).unwrap();
    fs::write(root.path().join("etc/modprobe.d/full.conf"), &content).unwrap();

    assert_eq!(
        fetch(root.path(), "/etc/modprobe.d/full.conf"),
        FetchAnswer::Text {
            path: path("/etc/modprobe.d/full.conf"),
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
