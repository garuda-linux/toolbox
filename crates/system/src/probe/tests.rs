mod boot;
mod fakes;
mod graph;
mod modules;

use std::collections::BTreeSet;
use std::sync::Arc;

use fakes::{
    FakeHost, FakeKernels, FakeLocales, FakePackages, FakeSettings, FakeSync, FakeUnits, sources,
};
use toolbox_plan::{
    ConfigWord, GroupName, HomePath, PackageName, UnitName, UserConfigEdit, UserName, WifiBackend,
};
use toolbox_state::accounts::{Accounts, CurrentUser, Group, User};
use toolbox_state::kernels::{KernelReleases, LinuxPackages, RunningKernel};
use toolbox_state::locales::{CurrentSystemLocale, EnabledLocales, SupportedLocales};
use toolbox_state::packages::PackageGraph;
use toolbox_state::parts::{
    AvailableShells, CurrentShells, EnabledUnits, EnabledUserUnits, InstalledPackages, MaskedUnits,
    MaskedUserUnits, SyncPackages, UserConfigStates,
};
use toolbox_state::settings::{CurrentDns, CurrentWifiBackend, GarudaUpdateConfig, HblockState};

use super::{Host, Probe, ProbeIssue, Sources, probe_snapshot};
use crate::ProbeError;

fn nico() -> UserName {
    UserName::new("nico").unwrap()
}

fn installed() -> InstalledPackages {
    InstalledPackages(BTreeSet::from([PackageName::new("bash").unwrap()]))
}

fn offered() -> SyncPackages {
    SyncPackages(BTreeSet::from([PackageName::new("steam").unwrap()]))
}

fn enabled() -> EnabledUnits {
    EnabledUnits(BTreeSet::from([UnitName::new("sshd.service").unwrap()]))
}

fn enabled_user() -> EnabledUserUnits {
    EnabledUserUnits(BTreeSet::from([UnitName::new("pipewire.socket").unwrap()]))
}

fn masked() -> MaskedUnits {
    MaskedUnits(BTreeSet::from([UnitName::new("cups.service").unwrap()]))
}

fn masked_user() -> MaskedUserUnits {
    MaskedUserUnits(BTreeSet::from([UnitName::new("tracker.service").unwrap()]))
}

fn units() -> FakeUnits {
    FakeUnits {
        system: Ok(enabled()),
        user: Ok(enabled_user()),
        masked: Ok(masked()),
        masked_user: Ok(masked_user()),
    }
}

fn accounts() -> Accounts {
    Accounts {
        users: vec![User {
            name: nico(),
            uid: 1000,
            gid: 1000,
            gecos: "Nico".to_owned(),
            home: "/home/nico".to_owned(),
            shell: "/usr/bin/fish".to_owned(),
        }],
        groups: vec![Group {
            name: GroupName::new("wheel").unwrap(),
            gid: 998,
            members: BTreeSet::from([nico()]),
        }],
    }
}

fn host() -> FakeHost {
    FakeHost {
        user: Ok(nico()),
        live_iso: Ok(true),
        desktops: Ok(vec!["KDE".to_owned()]),
        edition: Ok(Some("dr460nized".to_owned())),
        reboot_pending: Ok(true),
    }
}

fn booted() -> RunningKernel {
    RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.14.0-arch1-1").unwrap(),
        package: Some(PackageName::new("linux").unwrap()),
    }
}

fn kernels() -> FakeKernels {
    FakeKernels {
        linux: Ok(LinuxPackages::default()),
        running: Ok(booted()),
    }
}

#[test]
fn every_part_is_filled_when_all_probes_answer() {
    let (snapshot, issues) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));

    assert_eq!(issues, Vec::new());
    assert_eq!(snapshot.part::<InstalledPackages>(), Some(&installed()));
    assert_eq!(snapshot.part::<SyncPackages>(), Some(&offered()));
    assert_eq!(snapshot.part::<EnabledUnits>(), Some(&enabled()));
    assert_eq!(snapshot.part::<EnabledUserUnits>(), Some(&enabled_user()));
    assert_eq!(snapshot.part::<MaskedUnits>(), Some(&masked()));
    assert_eq!(snapshot.part::<MaskedUserUnits>(), Some(&masked_user()));
    assert_eq!(snapshot.part::<Accounts>(), Some(&accounts()));
    assert_eq!(snapshot.part::<CurrentUser>(), Some(&CurrentUser(nico())));
    assert_eq!(
        snapshot.part::<Host>(),
        Some(&Host {
            user: Some(nico()),
            live_iso: true,
            desktops: vec!["KDE".to_owned()],
            edition: Some("dr460nized".to_owned()),
            reboot_pending: true,
        })
    );
}

#[test]
fn a_missing_package_manager_leaves_only_its_parts_out() {
    let (snapshot, issues) =
        probe_snapshot(&sources(FakePackages(Err(ProbeError::Unavailable)), host()));

    assert_eq!(
        issues,
        vec![
            ProbeIssue {
                probe: Probe::Packages,
                error: ProbeError::Unavailable,
            },
            ProbeIssue {
                probe: Probe::PackageGraph,
                error: ProbeError::Unavailable,
            }
        ]
    );
    assert_eq!(snapshot.part::<InstalledPackages>(), None);
    assert_eq!(snapshot.part::<PackageGraph>(), None);
    assert_eq!(snapshot.part::<EnabledUnits>(), Some(&enabled()));
    assert_eq!(snapshot.part::<Accounts>(), Some(&accounts()));
    assert_eq!(snapshot.part::<CurrentUser>(), Some(&CurrentUser(nico())));
}

/// Never synced databases hide no catalog entry, they only leave the
/// sync part out.
#[test]
fn unsynced_databases_leave_only_the_sync_part_out() {
    let sources = Sources {
        sync: Arc::new(FakeSync(Err(ProbeError::Unavailable))),
        ..sources(FakePackages(Ok(installed())), host())
    };

    let (snapshot, issues) = probe_snapshot(&sources);

    assert_eq!(
        issues,
        vec![ProbeIssue {
            probe: Probe::SyncPackages,
            error: ProbeError::Unavailable,
        }]
    );
    assert_eq!(snapshot.part::<SyncPackages>(), None);
    assert_eq!(snapshot.part::<InstalledPackages>(), Some(&installed()));
}

#[test]
fn without_a_user_only_the_current_user_part_is_missing() {
    let failed = ProbeError::Failed("USER is not a valid name".to_owned());
    let (snapshot, issues) = probe_snapshot(&sources(
        FakePackages(Ok(installed())),
        FakeHost {
            user: Err(failed.clone()),
            ..host()
        },
    ));

    assert_eq!(
        issues,
        vec![ProbeIssue {
            probe: Probe::User,
            error: failed,
        }]
    );
    assert_eq!(snapshot.part::<CurrentUser>(), None);
    assert_eq!(snapshot.part::<Accounts>(), Some(&accounts()));
    assert_eq!(
        snapshot.part::<Host>().map(|host| host.user.clone()),
        Some(None)
    );
}

#[test]
fn failed_host_facts_fall_back_and_are_reported() {
    let broken = || ProbeError::Failed("broken".to_owned());
    let (snapshot, issues) = probe_snapshot(&sources(
        FakePackages(Ok(installed())),
        FakeHost {
            user: Ok(nico()),
            live_iso: Err(broken()),
            desktops: Err(broken()),
            edition: Err(ProbeError::Unavailable),
            reboot_pending: Err(broken()),
        },
    ));

    let probes: Vec<Probe> = issues.iter().map(|issue| issue.probe).collect();

    assert_eq!(
        probes,
        vec![
            Probe::LiveIso,
            Probe::Desktops,
            Probe::Edition,
            Probe::RebootPending
        ]
    );
    assert_eq!(
        snapshot.part::<Host>(),
        Some(&Host {
            user: Some(nico()),
            ..Host::default()
        })
    );
}

#[test]
fn failing_mask_probes_leave_only_their_parts_out() {
    let broken = || ProbeError::Failed("no user manager".to_owned());
    let sources = Sources {
        units: Arc::new(FakeUnits {
            masked: Err(ProbeError::Unavailable),
            masked_user: Err(broken()),
            ..units()
        }),
        ..sources(FakePackages(Ok(installed())), host())
    };

    let (snapshot, issues) = probe_snapshot(&sources);

    assert_eq!(
        issues,
        vec![
            ProbeIssue {
                probe: Probe::MaskedUnits,
                error: ProbeError::Unavailable,
            },
            ProbeIssue {
                probe: Probe::MaskedUserUnits,
                error: broken(),
            }
        ]
    );
    assert_eq!(snapshot.part::<MaskedUnits>(), None);
    assert_eq!(snapshot.part::<MaskedUserUnits>(), None);
    assert_eq!(snapshot.part::<EnabledUnits>(), Some(&enabled()));
}

#[test]
fn settings_parts_are_filled_or_reported_one_by_one() {
    let sources = Sources {
        settings: Arc::new(FakeSettings {
            dns: Err(ProbeError::Unavailable),
            garuda_update: Ok(GarudaUpdateConfig::default()),
        }),
        ..sources(FakePackages(Ok(installed())), host())
    };

    let (snapshot, issues) = probe_snapshot(&sources);

    assert_eq!(
        issues,
        vec![ProbeIssue {
            probe: Probe::Dns,
            error: ProbeError::Unavailable,
        }]
    );
    assert_eq!(snapshot.part::<CurrentDns>(), None);
    assert_eq!(snapshot.part::<HblockState>(), Some(&HblockState(true)));
    assert_eq!(
        snapshot.part::<CurrentWifiBackend>(),
        Some(&CurrentWifiBackend(WifiBackend::Iwd))
    );
    assert_eq!(
        snapshot.part::<GarudaUpdateConfig>(),
        Some(&GarudaUpdateConfig::default())
    );
    assert_eq!(
        snapshot.part::<CurrentShells>(),
        Some(&CurrentShells::default())
    );
    assert_eq!(
        snapshot.part::<AvailableShells>(),
        Some(&AvailableShells::default())
    );
}

#[test]
fn the_config_edits_of_the_catalogs_are_probed() {
    let edit = UserConfigEdit::JsonFlag {
        path: HomePath::new(".config/discord/settings.json").unwrap(),
        key: ConfigWord::new("SKIP_HOST_UPDATE").unwrap(),
    };

    let sources =
        sources(FakePackages(Ok(installed())), host()).with_config_edits(vec![edit.clone()]);

    let (snapshot, issues) = probe_snapshot(&sources);

    assert_eq!(issues, Vec::new());
    assert_eq!(
        snapshot.part::<UserConfigStates>(),
        Some(&UserConfigStates(std::iter::once((edit, true)).collect()))
    );
}

#[test]
fn the_kernel_and_locale_parts_are_filled() {
    let (snapshot, issues) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));

    assert_eq!(issues, Vec::new());
    assert_eq!(
        snapshot.part::<LinuxPackages>(),
        Some(&LinuxPackages::default())
    );
    assert_eq!(snapshot.part::<RunningKernel>(), Some(&booted()));
    assert_eq!(
        snapshot.part::<KernelReleases>(),
        Some(&KernelReleases::default())
    );
    assert_eq!(
        snapshot.part::<SupportedLocales>(),
        Some(&SupportedLocales::default())
    );
    assert_eq!(
        snapshot.part::<EnabledLocales>(),
        Some(&EnabledLocales::default())
    );
    assert_eq!(
        snapshot.part::<CurrentSystemLocale>(),
        Some(&CurrentSystemLocale::default())
    );
}

/// A system without pacman or without generated locales still shows the
/// kernel that runs and every other page.
#[test]
fn missing_kernel_packages_and_locales_leave_only_their_parts_out() {
    let sources = Sources {
        linux: Arc::new(FakeKernels {
            linux: Err(ProbeError::Unavailable),
            running: Ok(booted()),
        }),
        locales: Arc::new(FakeLocales(Err(ProbeError::Unavailable))),
        ..sources(FakePackages(Ok(installed())), host())
    };

    let (snapshot, issues) = probe_snapshot(&sources);

    assert_eq!(
        issues,
        vec![
            ProbeIssue {
                probe: Probe::LinuxPackages,
                error: ProbeError::Unavailable,
            },
            ProbeIssue {
                probe: Probe::EnabledLocales,
                error: ProbeError::Unavailable,
            },
        ]
    );
    assert_eq!(snapshot.part::<LinuxPackages>(), None);
    assert_eq!(snapshot.part::<EnabledLocales>(), None);
    assert_eq!(snapshot.part::<RunningKernel>(), Some(&booted()));
    assert_eq!(snapshot.part::<InstalledPackages>(), Some(&installed()));
}
