use super::{ConfigFilePath, InvalidConfigFilePath};

#[test]
fn non_ascii_characters_are_refused() {
    assert_eq!(
        ConfigFilePath::new("/etc/schön.conf").unwrap_err(),
        InvalidConfigFilePath::InvalidCharacter('ö')
    );
    assert_eq!(
        ConfigFilePath::new("/etc/\u{202e}fstab").unwrap_err(),
        InvalidConfigFilePath::InvalidCharacter('\u{202e}')
    );
}

#[test]
fn a_file_directly_below_a_root_splits_into_the_root_and_its_name() {
    let path = ConfigFilePath::new("/etc/hostname").unwrap();

    assert_eq!(path.split(), ("etc", "hostname"));
}

#[test]
fn vpn_disk_signing_and_tls_secrets_are_never_read() {
    for text in [
        "/etc/wireguard/wg0.conf",
        "/etc/wireguard",
        "/etc/cryptsetup-keys.d/root.key",
        "/etc/pacman.d/gnupg/secring.gpg",
        "/etc/pacman.d/gnupg/private-keys-v1.d/a.key",
        "/etc/ppp/chap-secrets",
        "/etc/ppp/pap-secrets",
        "/etc/ppp/srp-secrets",
        "/etc/ssl/private/server.key",
    ] {
        assert_eq!(
            ConfigFilePath::new(text).unwrap_err(),
            InvalidConfigFilePath::Withheld(text.to_owned()),
            "{text}"
        );
    }

    ConfigFilePath::new("/etc/ppp/options").unwrap();
    ConfigFilePath::new("/etc/ssl/certs/ca-certificates.crt").unwrap();
}

/// `/etc/systemd/network` stays a folder the page edits, so its `.netdev`
/// files need a rule by name: they hold the `WireGuard` private keys.
#[test]
fn a_netdev_file_is_never_read_or_written() {
    for text in [
        "/etc/systemd/network/wg0.netdev",
        "/etc/systemd/network/30-vpn.netdev",
        "/etc/systemd/network/.netdev",
    ] {
        assert_eq!(
            ConfigFilePath::new(text).unwrap_err(),
            InvalidConfigFilePath::Withheld(text.to_owned()),
            "{text}"
        );
    }

    let network = ConfigFilePath::new("/etc/systemd/network").unwrap();

    network.join("wg0.netdev").unwrap_err();
}

/// An editor or package copy keeps the keys under another name, and a
/// folder named like the file holds it, so every part counts.
#[test]
fn a_copy_of_a_netdev_file_or_a_folder_named_so_is_withheld() {
    for text in [
        "/etc/systemd/network/wg0.netdev.bak",
        "/etc/systemd/network/wg0.netdev.pacsave",
        "/etc/systemd/network/wg0.netdev~",
        "/etc/systemd/network/wg0.netdev.d/keys.network",
        "/etc/systemd/network/wg0.netdev.d",
    ] {
        assert_eq!(
            ConfigFilePath::new(text).unwrap_err(),
            InvalidConfigFilePath::Withheld(text.to_owned()),
            "{text}"
        );
    }

    let network = ConfigFilePath::new("/etc/systemd/network").unwrap();

    network.join("wg0.netdev.bak").unwrap_err();
    assert!(network.join("wg0.network").unwrap().is_writable());
}
