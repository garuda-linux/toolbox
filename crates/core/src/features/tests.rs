//! The codes the UI compares, pinned: they are derived from the variant
//! names, so a renamed variant would change one without a word.

mod recheck;

use super::{boot, bootrepair, configs, discover, kernels, locales, network, services, users};

#[test]
fn the_codes_of_locales_refusal_stay() {
    for (value, code) in [
        (locales::Refusal::SystemLocale, "system-locale"),
        (locales::Refusal::LastLocale, "last-locale"),
        (locales::Refusal::NotEnabled, "not-enabled"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_kernels_refusal_stay() {
    for (value, code) in [
        (kernels::Refusal::Running, "running"),
        (kernels::Refusal::RunningUnknown, "running-unknown"),
        (kernels::Refusal::LastKernel, "last-kernel"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_bootrepair_refusal_stay() {
    for (value, code) in [
        (bootrepair::Refusal::NotOffered, "not-offered"),
        (bootrepair::Refusal::Unavailable, "unavailable"),
        (bootrepair::Refusal::UnknownSystem, "unknown-system"),
        (bootrepair::Refusal::InUse, "in-use"),
        (bootrepair::Refusal::UnknownTarget, "unknown-target"),
        (bootrepair::Refusal::UnknownLayout, "unknown-layout"),
        (bootrepair::Refusal::NoPassphrase, "empty"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_boot_refusal_stay() {
    for (value, code) in [
        (boot::Refusal::Unavailable, "unavailable"),
        (boot::Refusal::CustomCommandLine, "custom-command-line"),
        (boot::Refusal::TooManyParameters, "too-many-parameters"),
        (boot::Refusal::UnknownTheme, "unknown-theme"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_users_refusal_stay() {
    for (value, code) in [
        (users::Refusal::Unavailable, "unavailable"),
        (users::Refusal::UnknownUser, "unknown-user"),
        (users::Refusal::UnknownGroup, "unknown-group"),
        (users::Refusal::Root, "root"),
        (users::Refusal::CurrentUser, "current-user"),
        (users::Refusal::SystemAccount, "system-account"),
        (users::Refusal::LastAdmin, "last-admin"),
        (users::Refusal::LastAdminLock, "last-admin-lock"),
        (users::Refusal::LockStateUnknown, "lock-state-unknown"),
        (users::Refusal::PrimaryGroup, "primary-group"),
        (users::Refusal::SystemGroup, "system-group"),
        (users::Refusal::UserExists, "user-exists"),
        (users::Refusal::GroupExists, "group-exists"),
        (users::Refusal::ShellNotListed, "shell-not-listed"),
        (users::Refusal::UidInUse, "uid-in-use"),
        (users::Refusal::UidOutOfRange, "uid-out-of-range"),
        (users::Refusal::HomeTaken, "home-taken"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_network_refusal_stay() {
    assert_eq!(network::Refusal::OtherFile.as_str(), "other-file");
    assert_eq!(network::Refusal::Unavailable.as_str(), "unavailable");
}

#[test]
fn the_codes_of_network_driver_state_stay() {
    for (value, code) in [
        (network::DriverState::Loaded, "loaded"),
        (network::DriverState::Blacklisted, "blacklisted"),
        (network::DriverState::Available, "available"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_services_unit_order_stay() {
    for (value, code) in [
        (services::UnitOrder::Name, "name"),
        (services::UnitOrder::State, "state"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_discover_app_order_stay() {
    for (value, code) in [
        (discover::AppOrder::Name, "name"),
        (discover::AppOrder::Selected, "selected"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_users_listing_user_order_stay() {
    for (value, code) in [
        (users::listing::UserOrder::Name, "name"),
        (users::listing::UserOrder::Id, "id"),
        (users::listing::UserOrder::FullName, "full-name"),
        (users::listing::UserOrder::Home, "home"),
        (users::listing::UserOrder::Shell, "shell"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_users_listing_group_order_stay() {
    for (value, code) in [
        (users::listing::GroupOrder::Name, "name"),
        (users::listing::GroupOrder::Id, "id"),
        (users::listing::GroupOrder::Members, "members"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn the_codes_of_boot_flag_stay() {
    for (value, code) in [
        (boot::Flag::Quiet, "quiet"),
        (boot::Flag::Splash, "splash"),
        (boot::Flag::MitigationsOff, "mitigations-off"),
        (boot::Flag::SysRq, "sysrq"),
    ] {
        assert_eq!(value.as_str(), code);
    }
}

#[test]
fn a_missing_passphrase_has_the_code_of_an_empty_password() {
    assert_eq!(
        bootrepair::Refusal::NoPassphrase.as_str(),
        toolbox_plan::InvalidPassword::Empty.as_str()
    );
}

#[test]
fn the_kinds_of_config_rows_reach_the_ui_in_kebab_case() {
    let kinds = [
        configs::RowKind::Section,
        configs::RowKind::File,
        configs::RowKind::Folder,
        configs::RowKind::Child,
    ];

    assert_eq!(
        kinds.map(<&str>::from),
        ["section", "file", "folder", "child"]
    );
}
