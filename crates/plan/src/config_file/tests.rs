use super::path::WITHHELD_CONFIG_FILES;
use super::{
    CONFIG_ROOTS, ConfigFilePath, ConfigText, ConfigWrite, FetchAnswer, FetchRefusal, Fingerprint,
    InvalidConfigFilePath, InvalidConfigText, MAX_CONFIG_BYTES,
};
use crate::action::testing::shown;
use crate::protocol::{MAX_LINE_BYTES, Mode, Request, encode};
use crate::{RootAction, Step, StepId};

fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

#[test]
fn files_below_the_roots_are_accepted() {
    for text in [
        "/etc/fstab",
        "/etc/default/grub",
        "/etc/X11/xorg.conf.d/20-gpu.conf",
        "/etc/garuda/garuda-update/config",
        "/boot/grub/grub.cfg",
        "/usr/share/icons/default/index.theme",
        "/usr/share/grub/themes/garuda-dr460nized/theme.txt",
        "/etc/modprobe.d",
        "/etc/a file with blanks.conf",
    ] {
        assert_eq!(path(text).as_str(), text);
    }
}

#[test]
fn anything_outside_the_roots_is_refused() {
    for text in [
        "/etc",
        "/home/user/.bashrc",
        "/root/.ssh/authorized_keys",
        "/usr/bin/sudo",
        "/usr/share/icons/hicolor/index.theme",
        "/boot/vmlinuz-linux",
        "/boot/grubx/grub.cfg",
        "/etcetera/file",
        "/var/lib/garuda-toolbox/backups/etc/fstab",
        "/proc/self/environ",
    ] {
        assert_eq!(
            ConfigFilePath::new(text).unwrap_err(),
            InvalidConfigFilePath::OutsideRoots(text.to_owned()),
            "{text}"
        );
    }
}

#[test]
fn a_path_cannot_climb_out_of_a_root() {
    for text in [
        "/etc/../root/.bashrc",
        "/etc/./fstab",
        "/etc//fstab",
        "/etc/fstab/",
        "/etc/..",
        "/boot/grub/../../etc/shadow",
        "//etc/fstab",
        "/",
    ] {
        assert_eq!(
            ConfigFilePath::new(text).unwrap_err(),
            InvalidConfigFilePath::InvalidComponent(text.to_owned()),
            "{text}"
        );
    }
}

#[test]
fn relative_empty_overlong_and_control_characters_are_refused() {
    assert_eq!(
        ConfigFilePath::new("").unwrap_err(),
        InvalidConfigFilePath::Empty
    );
    assert_eq!(
        ConfigFilePath::new("etc/fstab").unwrap_err(),
        InvalidConfigFilePath::Relative("etc/fstab".to_owned())
    );
    assert_eq!(
        ConfigFilePath::new("~/.bashrc").unwrap_err(),
        InvalidConfigFilePath::Relative("~/.bashrc".to_owned())
    );
    assert_eq!(
        ConfigFilePath::new("/etc/fs\ntab").unwrap_err(),
        InvalidConfigFilePath::InvalidCharacter('\n')
    );
    assert_eq!(
        ConfigFilePath::new("/etc/fs\0tab").unwrap_err(),
        InvalidConfigFilePath::InvalidCharacter('\0')
    );
    assert_eq!(
        ConfigFilePath::new(format!("/etc/{}", "a/".repeat(600))).unwrap_err(),
        InvalidConfigFilePath::TooLong {
            max: super::path::MAX_PATH_BYTES
        }
    );
    assert_eq!(
        ConfigFilePath::new(format!("/etc/{}", "a".repeat(256))).unwrap_err(),
        InvalidConfigFilePath::InvalidComponent(format!("/etc/{}", "a".repeat(256)))
    );
}

#[test]
fn files_that_hand_out_secrets_never_pass() {
    for text in WITHHELD_CONFIG_FILES {
        assert_eq!(
            ConfigFilePath::new(text).unwrap_err(),
            InvalidConfigFilePath::Withheld(text.to_owned())
        );
        assert!(CONFIG_ROOTS.iter().any(|root| text.starts_with(root)));
    }

    serde_json::from_str::<ConfigFilePath>("\"/etc/shadow\"").unwrap_err();

    for path in [
        "/etc/shadow-",
        "/etc/shadow.pacnew",
        "/etc/gshadow.pacsave",
        "/etc/gshadow.lock",
        "/etc/security/opasswd.old",
        "/etc/ssh/ssh_host_ed25519_key",
        "/etc/NetworkManager/system-connections/Garuda.nmconnection",
        "/etc/wpa_supplicant/wpa_supplicant.conf",
        "/etc/ipsec.d/private/key.pem",
    ] {
        assert_eq!(
            ConfigFilePath::new(path).unwrap_err(),
            InvalidConfigFilePath::Withheld(path.to_owned()),
        );
    }

    for path in ["/etc/ssh/sshd_config", "/etc/security/limits.conf"] {
        ConfigFilePath::new(path).unwrap();
    }
}

#[test]
fn a_file_of_a_folder_is_one_plain_name() {
    let folder = path("/etc/modprobe.d");

    assert_eq!(
        folder.join("nvidia.conf").unwrap().as_str(),
        "/etc/modprobe.d/nvidia.conf"
    );

    for name in ["", ".", "..", "a/b", "../shadow", "x\ny"] {
        folder.join(name).unwrap_err();
    }
}

#[test]
fn a_path_splits_into_folder_name_and_backup() {
    let grub = path("/etc/default/grub");

    assert_eq!(grub.relative(), "etc/default/grub");
    assert_eq!(grub.split(), ("etc/default", "grub"));
    assert_eq!(
        grub.backup(),
        "var/lib/garuda-toolbox/backups/etc/default/grub"
    );
    assert_eq!(grub.to_string(), "/etc/default/grub");
}

#[test]
fn a_path_is_checked_again_when_it_is_decoded() {
    assert_eq!(
        serde_json::from_str::<ConfigFilePath>("\"/etc/fstab\"").unwrap(),
        path("/etc/fstab")
    );
    serde_json::from_str::<ConfigFilePath>("\"/etc/../home/x\"").unwrap_err();
    serde_json::from_str::<ConfigFilePath>("\"/home/x\"").unwrap_err();
    assert_eq!(
        serde_json::to_string(&path("/etc/fstab")).unwrap(),
        "\"/etc/fstab\""
    );
}

#[test]
fn text_is_utf8_without_nul_and_has_a_size_limit() {
    assert_eq!(ConfigText::new("a = 1\n").unwrap().as_str(), "a = 1\n");
    assert_eq!(ConfigText::new("").unwrap().as_str(), "");
    assert_eq!(
        ConfigText::from_bytes("grüße\n".as_bytes().to_vec())
            .unwrap()
            .as_str(),
        "grüße\n"
    );
    assert_eq!(
        ConfigText::new("a\0b").unwrap_err(),
        InvalidConfigText::NotText
    );
    assert_eq!(
        ConfigText::from_bytes(vec![0x7f, b'E', b'L', b'F', 0xff, 0xfe]).unwrap_err(),
        InvalidConfigText::NotText
    );
    assert_eq!(
        ConfigText::new("x".repeat(MAX_CONFIG_BYTES + 1)).unwrap_err(),
        InvalidConfigText::TooLarge {
            max: MAX_CONFIG_BYTES
        }
    );
    assert_eq!(
        ConfigText::from_bytes(vec![b'x'; MAX_CONFIG_BYTES + 1]).unwrap_err(),
        InvalidConfigText::TooLarge {
            max: MAX_CONFIG_BYTES
        }
    );
    ConfigText::new("x".repeat(MAX_CONFIG_BYTES)).unwrap();
    serde_json::from_str::<ConfigText>("\"a\\u0000b\"").unwrap_err();
}

#[test]
fn the_debug_form_of_a_text_holds_no_content() {
    let text = ConfigText::new("password = hunter2\n").unwrap();

    assert_eq!(format!("{text:?}"), "ConfigText { bytes: 19 }");
}

#[test]
fn a_fingerprint_tells_versions_apart() {
    let one = Fingerprint::of(b"a = 1\n");

    assert_eq!(one, Fingerprint::of(b"a = 1\n"));
    assert_eq!(one, ConfigText::new("a = 1\n").unwrap().fingerprint());
    assert_ne!(one, Fingerprint::of(b"a = 2\n"));
    assert_ne!(one, Fingerprint::of(b"a = 1\n\n"));
    assert_ne!(Fingerprint::of(b""), Fingerprint::of(b"\0"));

    let decoded: Fingerprint = serde_json::from_str(&serde_json::to_string(&one).unwrap()).unwrap();

    assert_eq!(decoded, one);
}

#[test]
fn a_fetch_answer_is_one_tagged_line() {
    let text = FetchAnswer::Text {
        path: path("/etc/fstab"),
        content: ConfigText::new("a\n").unwrap(),
    };

    let line = "{\"result\":\"text\",\"path\":\"/etc/fstab\",\"content\":\"a\\n\"}";

    assert_eq!(encode(&text).unwrap(), format!("{line}\n"));
    assert_eq!(FetchAnswer::parse(line).unwrap(), text);
    assert_eq!(
        FetchAnswer::parse("{\"result\":\"refused\",\"reason\":\"not-text\"}").unwrap(),
        FetchAnswer::Refused {
            reason: FetchRefusal::NotText
        }
    );
    FetchAnswer::parse(&line.replace("a\\n", "a\\u0000")).unwrap_err();
    FetchAnswer::parse("{\"result\":\"text\",\"content\":\"a\"}").unwrap_err();
    FetchAnswer::parse("{\"result\":\"refused\",\"reason\":\"whatever\"}").unwrap_err();
    FetchAnswer::parse("").unwrap_err();
}

/// The path names the file a link led to, so the app must be able to trust
/// it like one of its own.
#[test]
fn a_fetch_answer_names_only_a_file_the_toolbox_edits() {
    for outside in [
        "/nix/store/x-fstab",
        "/etc/shadow",
        "/run/systemd/resolve/stub-resolv.conf",
        "/usr/lib/os-release",
        "/proc/self/mounts",
        "etc/fstab",
    ] {
        FetchAnswer::parse(&format!(
            "{{\"result\":\"text\",\"path\":\"{outside}\",\"content\":\"a\"}}"
        ))
        .unwrap_err();
    }
}

#[test]
fn refusals_have_stable_codes() {
    let codes = [
        (FetchRefusal::Missing, "missing"),
        (FetchRefusal::Link, "link"),
        (FetchRefusal::Dangling, "dangling"),
        (FetchRefusal::Outside, "outside"),
        (FetchRefusal::NotRegular, "not-regular"),
        (FetchRefusal::TooLarge, "too-large"),
        (FetchRefusal::NotText, "not-text"),
    ];

    for (refusal, code) in codes {
        assert_eq!(refusal.as_str(), code);
        assert_eq!(
            serde_json::to_string(&refusal).unwrap(),
            format!("\"{code}\"")
        );
    }
}

fn write(path_text: &str, content: &str, original: &[u8]) -> RootAction {
    RootAction::WriteConfigFile {
        write: ConfigWrite {
            path: path(path_text),
            content: ConfigText::new(content).unwrap(),
            original: Fingerprint::of(original),
        },
    }
}

#[test]
fn the_write_action_shows_file_size_and_backup() {
    assert_eq!(
        shown(&write("/etc/fstab", "a = 1\n", b"old")),
        [
            "write /etc/fstab (6 bytes), the old one kept as /var/lib/garuda-toolbox/backups/etc/fstab  # only when it did not change meanwhile"
        ]
    );
}

#[test]
fn a_request_cannot_name_a_file_outside_the_roots() {
    let request = Request::new(
        Mode::Run,
        vec![Step {
            id: StepId::new(1),
            action: write("/etc/fstab", "a\n", b"a\n"),
        }],
    );

    let line = encode(&request).unwrap();

    assert_eq!(Request::parse(line.trim_end()).unwrap(), request);

    for evil in [
        "/etc/shadow",
        "/etc/../root/x",
        "/usr/bin/sudo",
        "etc/fstab",
    ] {
        Request::parse(line.trim_end().replace("/etc/fstab", evil).as_str()).unwrap_err();
    }
}

/// The worst case for the line: every byte a control character, which
/// JSON writes as six.
#[test]
fn the_largest_text_fits_one_request_line() {
    let content = "\u{1}".repeat(MAX_CONFIG_BYTES);
    let request = Request::new(
        Mode::Run,
        vec![Step {
            id: StepId::new(1),
            action: write("/etc/fstab", &content, b""),
        }],
    );

    let line = encode(&request).unwrap();

    assert!(line.len() > 6 * MAX_CONFIG_BYTES);
    assert!(line.len() <= MAX_LINE_BYTES);
    assert_eq!(Request::parse(line.trim_end()).unwrap(), request);
}

#[test]
fn texts_and_paths_parse_from_str_and_borrow_as_str() {
    let parsed: ConfigText = "a = 1\n".parse().unwrap();

    assert_eq!(parsed.as_ref(), "a = 1\n");
    assert_eq!(parsed.to_string(), "a = 1\n");
    assert!(
        !format!("{parsed:?}").contains("a = 1"),
        "debug prints the content"
    );
    "a\0b".parse::<ConfigText>().unwrap_err();

    let path: ConfigFilePath = "/etc/fstab".parse().unwrap();

    assert_eq!(path.as_ref(), "/etc/fstab");
    assert_eq!(path.to_string(), "/etc/fstab");
    "/home/x".parse::<ConfigFilePath>().unwrap_err();
}
