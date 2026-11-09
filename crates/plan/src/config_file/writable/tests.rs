use serde_json::json;

use super::{
    READABLE_CONFIG_FILES, READABLE_CONFIG_FOLDERS, WRITABLE_CONFIG_FILES, WRITABLE_CONFIG_FOLDERS,
};
use crate::{ConfigFilePath, ConfigText, ConfigWrite, Fingerprint};

fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

fn assert_never_written(text: &str) {
    assert!(!path(text).is_writable(), "{text}");

    let error = serde_json::from_value::<ConfigWrite>(wire(text))
        .unwrap_err()
        .to_string();

    assert!(error.contains("never written"), "{text}: {error}");
}

fn wire(path: &str) -> serde_json::Value {
    let text = ConfigText::new("a = 1\n").unwrap();

    json!({
        "path": path,
        "content": text.as_str(),
        "original": Fingerprint::of(b"a = 0\n"),
    })
}

#[test]
fn files_the_catalog_does_not_name_are_never_written() {
    for text in [
        "/etc/passwd-",
        "/etc/sudoers.d/x",
        "/etc/ld.so.preload",
        "/etc/profile.d/x.sh",
        "/etc/cron.daily/below/x",
        "/etc/cron.daily",
        "/etc/pam.d.conf",
        "/boot/grub/custom.cfg",
        "/usr/share/icons/default/cursors",
    ] {
        assert_never_written(text);
    }
}

#[test]
fn catalog_files_and_the_files_of_catalog_folders_are_written() {
    for text in [
        "/etc/fstab",
        "/etc/pacman.conf",
        "/boot/grub/grub.cfg",
        "/usr/share/grub/themes/garuda-dr460nized/theme.txt",
        "/etc/modprobe.d/nvidia.conf",
        "/etc/modprobe.d/any.conf",
        "/etc/systemd/network/10-wired.network",
        "/etc/sddm.conf.d/kde_settings.conf",
    ] {
        assert!(path(text).is_writable(), "{text}");

        let write = serde_json::from_value::<ConfigWrite>(wire(text)).unwrap();

        assert_eq!(write.path, path(text));
    }
}

#[test]
fn every_table_entry_is_a_valid_path() {
    for text in WRITABLE_CONFIG_FILES
        .iter()
        .chain(&WRITABLE_CONFIG_FOLDERS)
        .chain(&READABLE_CONFIG_FILES)
        .chain(&READABLE_CONFIG_FOLDERS)
    {
        ConfigFilePath::new(*text).unwrap();
    }

    assert!(WRITABLE_CONFIG_FILES.is_sorted());
    assert!(WRITABLE_CONFIG_FOLDERS.is_sorted());
    assert!(READABLE_CONFIG_FILES.is_sorted());
    assert!(READABLE_CONFIG_FOLDERS.is_sorted());
}

/// Secrets in `/etc` and `/boot/grub` that no deny list names: only the
/// allow list keeps them out.
#[test]
fn files_the_catalog_does_not_name_are_never_handed_out() {
    for text in [
        "/etc/openvpn/client/vpn.conf",
        "/etc/letsencrypt/live/example.org/privkey.pem",
        "/etc/netctl/wlan0-home",
        "/etc/ppp/peers/provider",
        "/etc/sssd/sssd.conf",
        "/etc/postfix/sasl_passwd",
        "/etc/tor/torrc",
        "/etc/davfs2/secrets",
        "/boot/grub/grubenv",
        "/etc/cron.daily/below/x",
        "/etc/pam.d.conf",
        "/etc/sudoers.d/x",
    ] {
        assert!(!path(text).is_readable(), "{text}");
    }
}

#[test]
fn what_the_page_edits_or_shows_is_handed_out() {
    for text in [
        "/etc/fstab",
        "/etc/modprobe.d/any.conf",
        "/etc/systemd/network/10-wired.network",
        "/etc/passwd",
        "/etc/sudoers",
        "/etc/pam.d/system-auth",
        "/etc/cron.daily/man-db",
        "/etc/systemd/system/backup.service",
        "/etc/udev/rules.d/99-x.rules",
        "/etc/pacman.d/hooks/x.hook",
    ] {
        assert!(path(text).is_readable(), "{text}");
    }
}

/// What hands out root or the accounts is shown on the page but never
/// written: the catalog marks it `read_only` and keeps it off the table.
#[test]
fn read_only_files_and_the_files_of_read_only_folders_are_never_written() {
    for text in [
        "/etc/passwd",
        "/etc/group",
        "/etc/sudoers",
        "/etc/pam.d/system-auth",
        "/etc/cron.d/0hourly",
        "/etc/cron.daily/man-db",
        "/etc/cron.hourly/x",
        "/etc/cron.monthly/x",
        "/etc/cron.weekly/x",
        "/etc/systemd/system/backup.service",
        "/etc/udev/rules.d/99-x.rules",
        "/etc/pacman.d/hooks/x.hook",
    ] {
        assert_never_written(text);
    }
}
