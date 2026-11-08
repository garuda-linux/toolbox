use proptest::prelude::*;

use super::{InvalidName, NameKind, Reason};
use crate::{
    Charset, DkmsModuleName, DkmsVersion, GroupName, KernelModule, KernelRelease, LocaleName,
    PackageName, UnitName, UserName,
};

#[test]
fn package_name_accepts_repository_names() {
    for name in [
        "linux-zen",
        "lib32-mesa",
        "python-pyqt6",
        "gtk4",
        "libc++",
        "dbus-broker-units",
        "R",
    ] {
        assert_eq!(PackageName::new(name).unwrap().as_str(), name);
    }
}

#[test]
fn package_name_rejects_option_like_and_hidden_names() {
    assert_eq!(
        PackageName::new("--overwrite").unwrap_err(),
        InvalidName::new(NameKind::Package, "--overwrite", Reason::InvalidStart('-'))
    );
    assert_eq!(
        PackageName::new(".hidden").unwrap_err().reason(),
        Reason::InvalidStart('.')
    );
}

#[test]
fn package_name_rejects_shell_characters() {
    assert_eq!(
        PackageName::new("vim;rm").unwrap_err().reason(),
        Reason::InvalidCharacter(';')
    );
    assert_eq!(
        PackageName::new("vim neovim").unwrap_err().reason(),
        Reason::InvalidCharacter(' ')
    );
}

#[test]
fn empty_names_are_rejected() {
    assert_eq!(PackageName::new("").unwrap_err().reason(), Reason::Empty);
    assert_eq!(UserName::new("").unwrap_err().reason(), Reason::Empty);
}

#[test]
fn unit_name_needs_a_known_type_suffix() {
    for name in [
        "docker.service",
        "cups.socket",
        "fstrim.timer",
        "getty@tty1.service",
        "dev-sda1.swap",
    ] {
        assert!(UnitName::new(name).is_ok(), "{name}");
    }

    assert_eq!(
        UnitName::new("docker").unwrap_err().reason(),
        Reason::UnknownUnitType
    );
    assert_eq!(
        UnitName::new(".service").unwrap_err().reason(),
        Reason::InvalidStart('.')
    );
    assert_eq!(
        UnitName::new("a/b.service").unwrap_err().reason(),
        Reason::InvalidCharacter('/')
    );
}

#[test]
fn wheel_and_sudo_are_the_admin_groups() {
    assert!(GroupName::new("wheel").unwrap().is_admin());
    assert!(GroupName::new("sudo").unwrap().is_admin());
    assert!(!GroupName::new("audio").unwrap().is_admin());
    assert!(!GroupName::new("sudoers").unwrap().is_admin());
}

#[test]
fn user_and_group_names_follow_shadow_rules() {
    UserName::new("nico").unwrap();
    UserName::new("_apt").unwrap();
    GroupName::new("wheel").unwrap();
    GroupName::new("docker-users").unwrap();
    assert_eq!(
        UserName::new("Nico").unwrap_err().reason(),
        Reason::InvalidStart('N')
    );
    assert_eq!(
        UserName::new("1abc").unwrap_err().reason(),
        Reason::InvalidStart('1')
    );
    assert_eq!(
        GroupName::new("a".repeat(33)).unwrap_err().reason(),
        Reason::TooLong { max: 32 }
    );
}

#[test]
fn locale_names_cover_glibc_spellings() {
    for name in ["en_US.UTF-8", "C.UTF-8", "sr_RS@latin", "be_BY.UTF-8@latin"] {
        assert!(LocaleName::new(name).is_ok(), "{name}");
    }

    assert_eq!(
        LocaleName::new("en US").unwrap_err().reason(),
        Reason::InvalidCharacter(' ')
    );
}

#[test]
fn a_dash_and_an_underscore_give_one_kernel_module() {
    let dash = KernelModule::new("iwl-wifi").unwrap();

    assert_eq!(dash, KernelModule::new("iwl_wifi").unwrap());
    assert_eq!(dash.as_str(), "iwl_wifi");
    assert_eq!(
        serde_json::from_str::<KernelModule>("\"mt76-usb\"").unwrap(),
        KernelModule::new("mt76_usb").unwrap()
    );
    assert_eq!(
        "snd-hda".parse::<KernelModule>().unwrap().as_str(),
        "snd_hda"
    );
}

#[test]
fn kernel_modules_allow_dash_and_underscore() {
    KernelModule::new("r8168").unwrap();
    KernelModule::new("iwl-mvm_x").unwrap();
    assert_eq!(
        KernelModule::new("-r").unwrap_err().reason(),
        Reason::InvalidStart('-')
    );
}

#[test]
fn character_sets_are_charmap_names() {
    for name in [
        "UTF-8",
        "ISO-8859-15",
        "EUC-JP",
        "KOI8-R",
        "CP1251",
        "BIG5-HKSCS",
        "GB18030",
    ] {
        assert!(Charset::new(name).is_ok(), "{name}");
    }

    assert_eq!(
        Charset::new("UTF-8 extra").unwrap_err().reason(),
        Reason::InvalidCharacter(' ')
    );
}

#[test]
fn dkms_names_and_kernel_releases_are_single_path_components() {
    for name in [
        "nvidia",
        "acpi_call",
        "vboxhost",
        "8812au",
        "openrazer-driver",
        "zfs",
    ] {
        assert!(DkmsModuleName::new(name).is_ok(), "{name}");
    }

    for version in [
        "550.90.07",
        "7.1.6_OSE",
        "1.2.2",
        "0.13.1.r30.gabc",
        "5.2.20~rc1",
    ] {
        assert!(DkmsVersion::new(version).is_ok(), "{version}");
    }

    for release in [
        "6.14.0-4-cachyos",
        "6.13.8-zen1-1-zen",
        "6.6.1-arch1-1",
        "6.12.9-1-lts",
    ] {
        assert!(KernelRelease::new(release).is_ok(), "{release}");
    }

    assert_eq!(
        DkmsModuleName::new("nvidia/550").unwrap_err().reason(),
        Reason::InvalidCharacter('/')
    );
    assert_eq!(
        DkmsVersion::new("-k").unwrap_err().reason(),
        Reason::InvalidStart('-')
    );
    assert_eq!(
        KernelRelease::new("..").unwrap_err().reason(),
        Reason::InvalidStart('.')
    );
    assert_eq!(
        KernelRelease::new("6.1 --force").unwrap_err().reason(),
        Reason::InvalidCharacter(' ')
    );
}

#[test]
fn names_serialize_as_plain_strings() {
    let name = PackageName::new("firefox").unwrap();
    assert_eq!(serde_json::to_string(&name).unwrap(), "\"firefox\"");
    let back: PackageName = serde_json::from_str("\"firefox\"").unwrap();
    assert_eq!(back, name);
}

#[test]
fn deserializing_an_invalid_name_fails() {
    let error = serde_json::from_str::<PackageName>("\"-Syu\"").unwrap_err();
    assert!(error.to_string().contains("package name"), "{error}");
}

#[test]
fn error_message_names_the_kind_value_and_reason() {
    let error = UnitName::new("docker").unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid unit name `docker`: it needs a unit type suffix such as `.service`"
    );
}

proptest! {
    #[test]
    fn valid_package_names_stay_valid_and_hostile_ones_are_refused(value in "[a-z0-9][a-z0-9@._+-]{0,30}") {
        let name = PackageName::new(&value).unwrap();

        prop_assert_eq!(name.as_str(), value.as_str());
        prop_assert_eq!(
            PackageName::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_unit_names_stay_valid_and_hostile_ones_are_refused(base in "[a-z][a-z0-9_.@-]{0,20}") {
        let value = format!("{base}.service");

        prop_assert!(UnitName::new(&value).is_ok());
        prop_assert_eq!(
            UnitName::new(format!("{base};reboot.service")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_user_names_stay_valid_and_hostile_ones_are_refused(value in "[a-z_][a-z0-9_-]{0,15}") {
        let name = UserName::new(&value).unwrap();

        prop_assert_eq!(name.as_str(), value.as_str());
        prop_assert_eq!(
            UserName::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_locale_names_stay_valid_and_hostile_ones_are_refused(value in "[a-z]{2,3}_[A-Z]{2}\\.[A-Z0-9-]{0,6}") {
        prop_assert!(LocaleName::new(&value).is_ok());
        prop_assert_eq!(
            LocaleName::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_kernel_modules_stay_valid_and_hostile_ones_are_refused(value in "[a-z0-9][a-z0-9_-]{0,20}") {
        let name = KernelModule::new(&value).unwrap();

        prop_assert_eq!(name.as_str(), value.replace('-', "_"));
        prop_assert_eq!(
            KernelModule::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_charsets_stay_valid_and_hostile_ones_are_refused(value in "[A-Z0-9][A-Z0-9-]{0,10}") {
        prop_assert!(Charset::new(&value).is_ok());
        prop_assert_eq!(
            Charset::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_dkms_modules_stay_valid_and_hostile_ones_are_refused(value in "[a-z0-9][a-z0-9_.-]{0,20}") {
        prop_assert!(DkmsModuleName::new(&value).is_ok());
        prop_assert_eq!(
            DkmsModuleName::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_dkms_versions_stay_valid_and_hostile_ones_are_refused(value in "[0-9][a-z0-9._+~-]{0,15}") {
        prop_assert!(DkmsVersion::new(&value).is_ok());
        prop_assert_eq!(
            DkmsVersion::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_kernel_releases_stay_valid_and_hostile_ones_are_refused(value in "[0-9][a-z0-9._+-]{0,20}") {
        prop_assert!(KernelRelease::new(&value).is_ok());
        prop_assert_eq!(
            KernelRelease::new(format!("{value};reboot")).unwrap_err().reason(),
            Reason::InvalidCharacter(';')
        );
    }

    #[test]
    fn valid_package_names_survive_a_json_round_trip(value in "[a-z0-9][a-z0-9@._+-]{0,30}") {
        let name = PackageName::new(value).unwrap();
        let json = serde_json::to_string(&name).unwrap();
        prop_assert_eq!(serde_json::from_str::<PackageName>(&json).unwrap(), name);
    }
}

/// The value is the caller's and can be megabytes of anything. A message
/// shows its start with every control character spelled out.
#[test]
fn an_invalid_name_shows_escaped_and_cut() {
    assert_eq!(
        PackageName::new("--overwrite").unwrap_err().to_string(),
        "invalid package name `--overwrite`: it must not start with `-`"
    );
    assert!(
        PackageName::new("a\nb\u{1b}[31m")
            .unwrap_err()
            .to_string()
            .starts_with("invalid package name `a\\nb\\u{1b}[31m`: ")
    );

    let shown = PackageName::new("é".repeat(4 << 20))
        .unwrap_err()
        .to_string();

    assert!(
        shown.starts_with(&format!("invalid package name `{}…`: ", "é".repeat(64))),
        "{}",
        shown.len()
    );
    assert!(shown.len() < 256, "{}", shown.len());
}

#[test]
fn mapper_names_have_their_kind() {
    assert_eq!(NameKind::Mapper.to_string(), "device-mapper name");
}

#[test]
fn names_read_as_their_text_and_parse_from_it() {
    use std::str::FromStr as _;

    let name = PackageName::new("linux-zen").unwrap();

    let text: &str = name.as_ref();

    assert_eq!(text, "linux-zen");
    assert_eq!(PackageName::from_str("linux-zen").unwrap(), name);
    assert_eq!(PackageName::try_from("linux-zen".to_owned()).unwrap(), name);
}
