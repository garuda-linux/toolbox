//! A Garuda install the parity commands run against, read from captures
//! where the part comes from text.

use std::collections::BTreeSet;

use toolbox_core::Snapshot;
use toolbox_core::domains::{
    Accounts, AvailableShells, BootLoader, CurrentGrub, CurrentSystemLocale, CurrentUser,
    EnabledLocales, Group, InstalledPackages, PlymouthState, SdBootKernel, SyncPackages, User,
};
use toolbox_plan::sdboot::{LoaderSettings, SdBootMenu};
use toolbox_plan::{
    GroupName, GrubState, KernelRelease, LocaleEntry, PackageName, PlymouthTheme, ShellPath,
    UnitName, UserName, system_lang,
};
use toolbox_state::boot::KernelCmdline;
use toolbox_state::kernels::{
    KernelReleases, LinuxPackages, LocalPackage, RepoPackage, parse_status,
};
use toolbox_state::locales::{SupportedLocales, parse_supported};
use toolbox_state::network::drivers::{ModuleFacts, loaded_modules, network_modules};
use toolbox_state::services::{ActiveState, FileState, UnitInfo};
use toolbox_system::Host;

use super::super::FakeSystem;

const DEFAULT_GRUB: &str = toolbox_plan::testing::grub::INSTALLED_DEFAULT_GRUB;
/// v4l2loopback is built for linux-garuda only.
const DKMS_STATUS: &str = toolbox_state::testing::kernels::dkms::ONE_KERNEL_MISSING_STDOUT;
const KERNEL_CMDLINE: &str = toolbox_state::testing::boot::KERNEL_CMDLINE;
const LOADER_CONF: &str = toolbox_plan::testing::sdboot::LOADER_CONF;
const LOCALE_CONF: &str = toolbox_plan::testing::locales::LOCALE_CONF;
const LOCALE_GEN: &str = toolbox_plan::testing::locales::LOCALE_GEN;
const MODULES_DEP: &str = toolbox_system::testing::network::modules::BARE_METAL_MODULES_DEP;
const PLYMOUTH_THEMES: &str = toolbox_plan::testing::boot::PLYMOUTH_THEMES;
const PROC_MODULES: &str = toolbox_system::testing::network::modules::BARE_METAL_PROC_MODULES;
const SUPPORTED: &str = toolbox_plan::testing::locales::SUPPORTED;

/// The kernels of the dkms capture, each with its headers.
const KERNELS: [(&str, &str); 2] = [
    ("linux-garuda", "7.3.1-1-garuda"),
    ("linux-lts", "6.18.60-1-lts"),
];

fn user(name: &str, uid: u32) -> User {
    User {
        name: UserName::new(name).unwrap(),
        uid,
        gid: uid,
        gecos: String::new(),
        home: format!("/home/{name}"),
        shell: "/usr/bin/bash".parse().unwrap(),
    }
}

fn group(name: &str, gid: u32, members: &[&str]) -> Group {
    Group {
        name: GroupName::new(name).unwrap(),
        gid,
        members: members
            .iter()
            .map(|member| UserName::new(*member).unwrap())
            .collect(),
    }
}

fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap()
}

fn unit(name: &str) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: String::new(),
        active: ActiveState::Inactive,
        sub_state: "dead".to_owned(),
        file_state: FileState::Disabled,
    }
}

fn accounts() -> Accounts {
    Accounts {
        users: vec![user("nico", 1000), user("ada", 1001), user("guest", 1002)],
        groups: vec![
            group("nico", 1000, &[]),
            group("ada", 1001, &[]),
            group("guest", 1002, &[]),
            group("media", 1003, &[]),
            group("wheel", 998, &["nico"]),
        ],
    }
}

fn kernel_packages() -> Vec<String> {
    KERNELS
        .iter()
        .flat_map(|(kernel, _release)| [(*kernel).to_owned(), format!("{kernel}-headers")])
        .collect()
}

fn linux_packages() -> LinuxPackages {
    let names = kernel_packages();

    LinuxPackages {
        sync: names
            .iter()
            .map(|name| RepoPackage {
                repo: "core".to_owned(),
                name: package(name),
                version: "1-1".to_owned(),
                description: String::new(),
            })
            .collect(),
        local: names
            .iter()
            .map(|name| LocalPackage {
                name: package(name),
                version: "1-1".to_owned(),
                description: String::new(),
            })
            .collect(),
    }
}

/// What every page reads, apart from the boot loader's parts.
fn install() -> Snapshot {
    let mut snapshot = Snapshot::default();
    let mut installed: BTreeSet<PackageName> =
        kernel_packages().iter().map(|name| package(name)).collect();

    installed.insert(package("pacman"));

    snapshot.insert(Host::default());
    snapshot.insert(accounts());
    snapshot.insert(CurrentUser(UserName::new("nico").unwrap()));
    snapshot.insert(AvailableShells(vec![
        ShellPath::new("/usr/bin/bash").unwrap(),
        ShellPath::new("/usr/bin/fish").unwrap(),
    ]));
    snapshot.insert(SyncPackages(
        installed
            .iter()
            .cloned()
            .chain([package("firefox")])
            .collect(),
    ));
    snapshot.insert(InstalledPackages(installed));
    snapshot.insert(linux_packages());
    snapshot.insert(KernelReleases(
        KERNELS
            .iter()
            .map(|(kernel, release)| (package(kernel), KernelRelease::new(*release).unwrap()))
            .collect(),
    ));
    snapshot.insert(PlymouthState {
        themes: PLYMOUTH_THEMES
            .lines()
            .map(|theme| PlymouthTheme::new(theme).unwrap())
            .collect(),
        current: Some(PlymouthTheme::new("dr460nized").unwrap()),
    });
    snapshot.insert(SupportedLocales(parse_supported(SUPPORTED)));
    snapshot.insert(EnabledLocales(LocaleEntry::enabled_in(LOCALE_GEN)));
    snapshot.insert(CurrentSystemLocale(system_lang(LOCALE_CONF)));
    snapshot.insert(ModuleFacts {
        loaded: loaded_modules(PROC_MODULES),
        modules: network_modules(MODULES_DEP),
        blacklists: Vec::new(),
    });
    snapshot
}

pub(super) fn grub_install() -> Snapshot {
    let mut snapshot = install();

    snapshot.insert(BootLoader::Grub);
    snapshot.insert(CurrentGrub(GrubState::read(DEFAULT_GRUB)));
    snapshot
}

/// Booted by systemd-boot with unified kernel images.
pub(super) fn sdboot_install() -> Snapshot {
    let mut snapshot = install();

    snapshot.insert(BootLoader::SystemdBoot);
    snapshot.insert(SdBootKernel {
        cmdline: KernelCmdline::read(KERNEL_CMDLINE, false),
        theme: None,
    });
    snapshot
}

/// The German locale enabled beside the capture's English one, so the
/// system locale can switch to it.
pub(super) fn german_enabled() -> Snapshot {
    let mut snapshot = grub_install();
    let locale_gen = format!("{LOCALE_GEN}de_DE.UTF-8 UTF-8\n");

    snapshot.insert(EnabledLocales(LocaleEntry::enabled_in(&locale_gen)));
    snapshot
}

/// What `queue add` reads beside the snapshot.
pub(super) fn reads() -> FakeSystem {
    FakeSystem {
        units: ["cups.service", "foo.service", "sshd.service"]
            .map(unit)
            .to_vec(),
        dkms: parse_status(DKMS_STATUS),
        menu: Some(SdBootMenu {
            settings: LoaderSettings::parse(LOADER_CONF),
            ..SdBootMenu::default()
        }),
    }
}
