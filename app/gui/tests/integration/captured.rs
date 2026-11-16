//! The system the readers answer with when a test gives none, from the Lima VM captures.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use egui_kittest::{Harness, HarnessBuilder};
use futures_lite::future::Boxed;
use toolbox_catalog::Check;
use toolbox_core::Snapshot;
use toolbox_core::domains::{
    AvailableShells, CurrentDns, CurrentGrub, CurrentShells, CurrentSystemLocale, CurrentUser,
    CurrentWifiBackend, EnabledLocales, EnabledUnits, EnabledUserUnits, GarudaUpdateConfig,
    HblockState, InstalledPackages, OwnBlacklist, SyncPackages, UserConfigStates,
};
use toolbox_core::features::kernels::parse_status;
use toolbox_core::features::locales::{SupportedLocales, parse_supported};
use toolbox_gui::boot::{BootSetup, Chosen, FileChooser, FileKind};
use toolbox_gui::news::SampleNews;
use toolbox_gui::units::{self, UnitList, UnitLister};
use toolbox_gui::users::UsersSetup;
use toolbox_gui::{App, Setup, reader, snapshot};
use toolbox_news::NewsSource as _;
use toolbox_plan::{
    DnsProvider, GarudaUpdateFlag, GrubMenu, GrubPlatform, GrubState, LocaleEntry, LocaleName,
    PackageName, UnitScope, UserConfigEdit, UserName, WifiBackend,
};
use toolbox_services::catalogs::Catalogs;
use toolbox_services::dkms::Dkms;
use toolbox_state::parts::{MaskedUnits, MaskedUserUnits};
use toolbox_system::systemd::{ListUnitsRow, UnitFileRow, unit_infos};
use toolbox_system::{
    BootMenuSource, CallError, Host, LockProblem, LockSource, MenuRead, parse_accounts,
    parse_available_shells, parse_login_shells,
};

use crate::common::{launch_with, packages, shared_runtime};

const PASSWD: &str = toolbox_plan::testing::accounts::PASSWD;
const GROUP: &str = toolbox_exec::testing::accounts::GROUP;
const SHELLS: &str = toolbox_exec::testing::accounts::SHELLS;
const LOCALE_GEN: &str = toolbox_plan::testing::locales::LOCALE_GEN;
const LOCALE_CONF: &str = toolbox_plan::testing::locales::LOCALE_CONF;
const SUPPORTED: &str = toolbox_plan::testing::locales::SUPPORTED;
const LIMA_DEFAULT_GRUB: &str = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;
const LIMA_GRUB_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;
const INSTALLED_DKMS: &str = toolbox_state::testing::kernels::dkms::INSTALLED_STDOUT;
const LIST_UNITS_JSON: &str = toolbox_system::testing::systemd::LIST_UNITS_JSON;
const LIST_UNIT_FILES_JSON: &str = toolbox_system::testing::systemd::LIST_UNIT_FILES_JSON;

/// The Lima VM's units, read through the probe's own parse of its replies.
struct CapturedUnits;

impl UnitLister for CapturedUnits {
    fn list(&self, scope: UnitScope) -> Boxed<UnitList> {
        let listed = match scope {
            UnitScope::System => Ok(Arc::new(unit_infos(unit_rows(), file_rows()))),
            UnitScope::User => Err(CallError::Unavailable),
        };

        Box::pin(std::future::ready(listed))
    }
}

/// The lock flags are left out, so no account shows as locked.
struct NoLocks;

impl LockSource for NoLocks {
    fn locks(&self) -> Result<BTreeMap<UserName, bool>, LockProblem> {
        Ok(BTreeMap::new())
    }
}

/// The Lima VM's GRUB menu, as the Boot options page reads it.
struct LimaMenu;

impl BootMenuSource for LimaMenu {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        MenuRead::Known(GrubMenu::for_platform(LIMA_GRUB_CFG, GrubPlatform::Efi))
    }
}

/// The dialog closes at once, so no picture waits for a file.
struct NoChooser;

impl FileChooser for NoChooser {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Chosen {
        Chosen::Cancelled
    }
}

/// A `busctl` reply is a `{"type", "data"}` envelope around one array.
fn reply(json: &str) -> serde_json::Value {
    let mut envelope: serde_json::Value = serde_json::from_str(json).expect("a capture");
    envelope["data"][0].take()
}

fn unit_rows() -> Vec<ListUnitsRow> {
    serde_json::from_value(reply(LIST_UNITS_JSON)).expect("the captured units")
}

fn file_rows() -> Vec<UnitFileRow> {
    serde_json::from_value(reply(LIST_UNIT_FILES_JSON)).expect("the captured unit files")
}

pub fn launch_covered(builder: HarnessBuilder<App>, setup: Setup) -> Harness<'static, App> {
    launch_with(builder, setup, covered)
}

/// The readers of the pages the other tests give none: status, facts,
/// news, dkms, units, users, boot and config files.
fn covered(mut setup: Setup, ctx: &egui::Context) -> Setup {
    let handle = shared_runtime().handle();
    let repaint = || toolbox_gui::waking::repaint(ctx.clone());
    let user = UserName::new("user").expect("a user name");
    setup.snapshot = Some(sample_reader(ctx, &setup.catalogs, Some(user)));

    Setup {
        status: Some(reader::spawn(
            handle,
            repaint(),
            "status",
            crate::home::quiet,
        )),
        facts: Some(reader::spawn(
            handle,
            repaint(),
            "facts",
            crate::home::quiet_facts,
        )),
        news: Some(reader::spawn_fallible(handle, repaint(), "news", || {
            SampleNews.news()
        })),
        dkms: Some(reader::spawn(handle, repaint(), "dkms", || {
            Dkms::Modules(parse_status(INSTALLED_DKMS))
        })),
        units: Some(units::spawn(
            handle,
            repaint(),
            Arc::new(CapturedUnits),
            Duration::ZERO,
        )),
        users: Some(UsersSetup {
            runtime: handle.clone(),
            locks: Arc::new(NoLocks),
        }),
        boot: Some(BootSetup {
            runtime: handle.clone(),
            menu: Arc::new(LimaMenu),
            chooser: Arc::new(NoChooser),
        }),
        configs: Some(crate::configs::read_only(handle)),
        ..setup
    }
}

/// The packages of every toggle in the catalogs. The sync databases offer
/// them all, so the toggle pages keep every row.
fn catalog_packages(catalogs: &Catalogs) -> BTreeSet<PackageName> {
    let mut names = BTreeSet::new();
    for toggles in [
        &catalogs.components,
        &catalogs.gaming,
        &catalogs.settings,
        &catalogs.setup,
    ] {
        let Ok(catalog) = &toggles.catalog else {
            continue;
        };

        for entry in catalog.entries() {
            if let Check::Package(name) = entry.check() {
                names.insert(name.clone());
            }
        }
    }

    names
}

/// A system that was read as the probe reads the Lima VM. Steam is
/// installed, the kernels of [`crate::kernels::system`] are too, and no
/// unit is enabled. The account, locale, GRUB and tweak parts come from the
/// captures or from the values the probe gives them.
pub fn sample_reader(
    ctx: &egui::Context,
    catalogs: &Catalogs,
    host_user: Option<UserName>,
) -> snapshot::SnapshotHandle {
    let synced = catalog_packages(catalogs);
    let edits = catalogs.config_edits();

    snapshot::spawn(
        shared_runtime().handle(),
        toolbox_gui::waking::repaint(ctx.clone()),
        move || {
            (
                sample_system(&synced, &edits, host_user.clone()),
                Vec::new(),
            )
        },
    )
}

fn sample_system(
    synced: &BTreeSet<PackageName>,
    edits: &[UserConfigEdit],
    host_user: Option<UserName>,
) -> Snapshot {
    let user = UserName::new("user").expect("a user name");
    let mut snapshot = crate::kernels::system(true);
    snapshot.insert(InstalledPackages(packages(&[
        "steam",
        "linux",
        "linux-headers",
        "linux-lts",
    ])));
    snapshot.insert(SyncPackages(synced.clone()));
    snapshot.insert(EnabledUnits(BTreeSet::new()));
    snapshot.insert(EnabledUserUnits(BTreeSet::new()));
    snapshot.insert(MaskedUnits(BTreeSet::new()));
    snapshot.insert(MaskedUserUnits(BTreeSet::new()));
    snapshot.insert(OwnBlacklist(BTreeSet::new()));
    snapshot.insert(UserConfigStates(
        edits.iter().map(|edit| (edit.clone(), false)).collect(),
    ));
    snapshot.insert(Host {
        user: host_user,
        ..Host::default()
    });
    snapshot.insert(CurrentUser(user));
    snapshot.insert(parse_accounts(PASSWD, GROUP));
    snapshot.insert(CurrentShells(parse_login_shells(PASSWD)));
    snapshot.insert(AvailableShells(parse_available_shells(SHELLS)));
    snapshot.insert(SupportedLocales(parse_supported(SUPPORTED)));
    snapshot.insert(EnabledLocales(LocaleEntry::enabled_in(LOCALE_GEN)));
    snapshot.insert(CurrentSystemLocale(system_locale(LOCALE_CONF)));
    snapshot.insert(CurrentGrub(GrubState::read(LIMA_DEFAULT_GRUB)));
    snapshot.insert(CurrentDns(DnsProvider::Default));
    snapshot.insert(CurrentWifiBackend(WifiBackend::WpaSupplicant));
    snapshot.insert(HblockState(false));
    snapshot.insert(GarudaUpdateConfig(BTreeSet::from([
        GarudaUpdateFlag::UpdateAur,
    ])));
    snapshot
}

fn system_locale(conf: &str) -> Option<LocaleName> {
    conf.lines()
        .find_map(|line| line.strip_prefix("LANG="))
        .and_then(|name| LocaleName::new(name).ok())
}
