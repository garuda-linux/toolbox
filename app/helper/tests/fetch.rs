//! `garuda-toolbox-helper fetch <path>`, the built program: which paths it
//! takes at all and that every answer is one line.

mod oneshot;

use oneshot::output as helper;
use toolbox_plan::{FetchAnswer, FetchRefusal};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The path is the caller's, so the helper must refuse on its own what
/// lies outside the folders the toolbox edits, before it touches a file.
#[test]
fn fetch_takes_only_paths_of_config_files() -> TestResult {
    for args in [
        vec!["fetch"],
        vec!["fetch", ""],
        vec!["fetch", "/etc/fstab", "extra"],
        vec!["fetch", "etc/fstab"],
        vec!["fetch", "/etc/shadow"],
        vec!["fetch", "/etc/gshadow"],
        vec!["fetch", "/etc/shadow-"],
        vec!["fetch", "/etc/../etc/shadow"],
        vec!["fetch", "/etc/../root/.ssh/id_ed25519"],
        vec!["fetch", "/etc/./fstab"],
        vec!["fetch", "/etc//fstab"],
        vec!["fetch", "/root/.bash_history"],
        vec!["fetch", "/home"],
        vec!["fetch", "/proc/self/environ"],
        vec!["fetch", "/var/lib/garuda-toolbox/backups/etc/sudoers"],
        vec!["fetch", "/boot/vmlinuz-linux"],
        vec!["fetch", "--help"],
        vec!["fetch", "shadow-locks"],
    ] {
        let output = helper(&args)?;

        assert_eq!(output.status.code(), Some(2_i32), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
    }

    Ok(())
}

/// Secrets below the config folders that no deny list names: only the
/// catalog's allow list keeps them out. The world-readable
/// `/etc/nsswitch.conf` of Arch and `NixOS` is refused like them, as the
/// refusal comes before any look at the file.
#[test]
fn fetch_takes_no_file_the_catalog_does_not_name() -> TestResult {
    for path in [
        "/etc/nsswitch.conf",
        "/etc/openvpn/client/vpn.conf",
        "/etc/letsencrypt/live/example.org/privkey.pem",
        "/etc/netctl/wlan0-home",
        "/etc/ppp/peers/provider",
        "/etc/sssd/sssd.conf",
        "/etc/postfix/sasl_passwd",
        "/etc/tor/torrc",
        "/etc/davfs2/secrets",
        "/etc/systemd/network/wg0.netdev",
        "/boot/grub/grubenv",
        "/etc/garuda-toolbox-test/no-such-file.conf",
    ] {
        let output = helper(&["fetch", path])?;

        assert_eq!(output.status.code(), Some(2_i32), "{path}");
        assert!(output.stdout.is_empty(), "{path}");
    }

    Ok(())
}

#[test]
fn a_missing_file_is_answered_on_one_line() -> TestResult {
    let output = helper(&["fetch", "/etc/modprobe.d/garuda-toolbox-test-missing.conf"])?;
    let answer = String::from_utf8(output.stdout)?;

    assert_eq!(output.status.code(), Some(0_i32));
    assert_eq!(answer, "{\"result\":\"refused\",\"reason\":\"missing\"}\n");
    Ok(())
}

/// `/etc/passwd` is a regular, world readable text file on Arch and on
/// `NixOS`, so the read works without root. Where it is something else
/// the helper still answers one line.
#[test]
fn a_readable_file_is_answered_as_text() -> TestResult {
    let output = helper(&["fetch", "/etc/passwd"])?;
    let answer = String::from_utf8(output.stdout)?;

    assert_eq!(output.status.code(), Some(0_i32));
    assert_eq!(answer.matches('\n').count(), 1);

    match FetchAnswer::parse(answer.trim_end())? {
        FetchAnswer::Text { path, content } => {
            assert_eq!(path.as_str(), "/etc/passwd");
            assert!(content.as_str().contains("root:"));
        }
        FetchAnswer::Refused { reason } => assert!(
            matches!(reason, FetchRefusal::Outside | FetchRefusal::Missing),
            "{}",
            reason.as_str()
        ),
    }

    Ok(())
}

/// `/etc/os-release` is a link to a package's file in `/usr/lib` on Arch
/// and into `/nix/store` on `NixOS`: both are followed and neither is
/// handed out.
#[test]
fn a_link_to_a_package_file_is_refused_on_one_line() -> TestResult {
    if !std::fs::symlink_metadata("/etc/os-release").is_ok_and(|file| file.is_symlink()) {
        return Ok(());
    }

    let output = helper(&["fetch", "/etc/os-release"])?;

    assert_eq!(output.status.code(), Some(0_i32));
    assert_eq!(
        String::from_utf8(output.stdout)?,
        "{\"result\":\"refused\",\"reason\":\"outside\"}\n"
    );
    Ok(())
}
