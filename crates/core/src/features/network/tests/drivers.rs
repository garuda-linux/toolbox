use std::collections::BTreeSet;

use toolbox_plan::{KernelModule, PlannedAction};

use crate::domains::{ModuleBlacklist, OwnBlacklist};
use crate::features::network::{
    BlacklistEntry, Category, DriverFilter, DriverRow, DriverState, ModuleFacts, Refusal,
    driver_rows, loaded_modules, network_modules, plan_load, plan_unload, set_blacklisted, visible,
};
use crate::testing::commands;
use crate::{ChangeSet, Snapshot, Toggle};

pub(super) const OWN_FILE: &str = "/etc/modprobe.d/garuda-toolbox-blacklist.conf";
pub(super) const BROADCOM_FILE: &str = "/usr/lib/modprobe.d/broadcom-wl.conf";

const MODULES_DEP: &str = "\
kernel/arch/x86/kvm/kvm.ko.zst: kernel/virt/lib/irqbypass.ko.zst
kernel/drivers/net/wireless/intel/iwlwifi/iwlwifi.ko.zst: kernel/net/wireless/cfg80211.ko.zst
kernel/drivers/net/wireless/mediatek/mt76/mt76-usb.ko.zst: kernel/net/mac80211/mac80211.ko.zst
kernel/drivers/net/wireless/broadcom/b43/b43.ko.zst:
kernel/drivers/net/ethernet/realtek/r8169.ko.zst: kernel/drivers/net/phy/realtek/realtek.ko.zst
kernel/drivers/net/usb/r8152.ko.zst: kernel/drivers/net/mii.ko.zst
kernel/drivers/net/mii.ko:
kernel/net/wireless/cfg80211.ko.zst: kernel/net/rfkill/rfkill.ko.zst
updates/dkms/wl.ko.zst: kernel/net/wireless/cfg80211.ko.zst
updates/dkms/nvidia.ko.zst:
";

const PROC_MODULES: &str = "\
iwlwifi 602112 1 iwlmvm, Live 0x0000000000000000
mt76_usb 49152 1 mt7921u, Live 0x0000000000000000
kvm 1404928 1 kvm_intel, Live 0x0000000000000000
r8169 135168 0 - Live 0x0000000000000000
";

pub(super) fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

pub(super) fn entry(name: &str, file: &str) -> BlacklistEntry {
    BlacklistEntry {
        module: module(name),
        file: file.to_owned(),
        own: file == OWN_FILE,
    }
}

pub(super) fn facts(blacklists: Vec<BlacklistEntry>) -> ModuleFacts {
    ModuleFacts {
        loaded: loaded_modules(PROC_MODULES),
        modules: network_modules(MODULES_DEP),
        blacklists,
    }
}

fn snapshot(own: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(OwnBlacklist(
        own.iter().map(|name| module(name)).collect::<BTreeSet<_>>(),
    ));
    snapshot
}

fn row<'rows>(rows: &'rows [DriverRow], name: &str) -> &'rows DriverRow {
    rows.iter()
        .find(|row| row.name.as_str() == name)
        .unwrap_or_else(|| panic!("no row {name}"))
}

fn names(rows: &[&DriverRow]) -> Vec<String> {
    rows.iter().map(|row| row.name.to_string()).collect()
}

#[test]
fn network_modules_come_from_the_drivers_folder_and_the_known_few_elsewhere() {
    let found: Vec<(String, Category)> = network_modules(MODULES_DEP)
        .into_iter()
        .map(|module| (module.name.to_string(), module.category))
        .collect();

    assert_eq!(
        found,
        [
            ("b43".to_owned(), Category::Wireless),
            ("iwlwifi".to_owned(), Category::Wireless),
            ("mii".to_owned(), Category::Other),
            ("mt76_usb".to_owned(), Category::Wireless),
            ("r8152".to_owned(), Category::Other),
            ("r8169".to_owned(), Category::Ethernet),
            ("wl".to_owned(), Category::Wireless),
        ]
    );
}

#[test]
fn loaded_modules_are_the_first_words_of_proc_modules() {
    let loaded: Vec<String> = loaded_modules(PROC_MODULES)
        .into_iter()
        .map(|module| module.to_string())
        .collect();

    assert_eq!(loaded, ["iwlwifi", "kvm", "mt76_usb", "r8169"]);
    assert_eq!(loaded_modules(""), BTreeSet::new());
}

#[test]
fn a_module_with_a_dash_in_its_file_name_shows_as_loaded() {
    let rows = driver_rows(&facts(Vec::new()), &ChangeSet::default());

    assert_eq!(row(&rows, "mt76_usb").state, DriverState::Loaded);
}

#[test]
fn rows_are_loaded_first_then_blacklisted_then_available() {
    let rows = driver_rows(
        &facts(vec![entry("b43", BROADCOM_FILE), entry("wl", OWN_FILE)]),
        &ChangeSet::default(),
    );

    let shown: Vec<(&str, &str)> = rows
        .iter()
        .map(|row| (row.name.as_str(), row.state.as_str()))
        .collect();

    assert_eq!(
        shown,
        [
            ("iwlwifi", "loaded"),
            ("mt76_usb", "loaded"),
            ("r8169", "loaded"),
            ("b43", "blacklisted"),
            ("wl", "blacklisted"),
            ("mii", "available"),
            ("r8152", "available"),
        ]
    );
    assert!(rows.iter().all(|row| row.name.as_str() != "kvm"));
}

#[test]
fn a_blacklist_of_another_file_names_the_file() {
    let rows = driver_rows(
        &facts(vec![
            entry("b43", BROADCOM_FILE),
            entry("b43", "/etc/modprobe.d/blacklist.conf"),
            entry("nouveau", "/usr/lib/modprobe.d/nvidia.conf"),
        ]),
        &ChangeSet::default(),
    );

    let b43 = row(&rows, "b43");

    assert!(!b43.own);
    assert_eq!(
        b43.other_files,
        [BROADCOM_FILE, "/etc/modprobe.d/blacklist.conf"]
    );
    assert!(
        rows.iter().all(|row| row.name.as_str() != "nouveau"),
        "a blacklisted module that is no network module shows"
    );
}

#[test]
fn a_loaded_module_keeps_its_blacklist_mark() {
    let rows = driver_rows(
        &facts(vec![entry("r8169", OWN_FILE)]),
        &ChangeSet::default(),
    );

    let r8169 = row(&rows, "r8169");

    assert_eq!(r8169.state, DriverState::Loaded);
    assert!(r8169.loaded && r8169.own);
}

#[test]
fn whatever_the_own_file_lists_shows_so_it_can_be_taken_off() {
    let rows = driver_rows(
        &facts(vec![entry("pcspkr", OWN_FILE)]),
        &ChangeSet::default(),
    );

    let pcspkr = row(&rows, "pcspkr");

    assert_eq!(pcspkr.state, DriverState::Blacklisted);
    assert_eq!(pcspkr.category, Category::Other);
    assert!(pcspkr.own);
}

#[test]
fn the_filter_narrows_by_state_category_and_name() {
    let rows = driver_rows(
        &facts(vec![entry("b43", BROADCOM_FILE)]),
        &ChangeSet::default(),
    );

    let filter = |search: &str, state, category| DriverFilter {
        search: search.to_owned(),
        state,
        category,
    };

    assert_eq!(visible(&rows, &DriverFilter::default()).len(), rows.len());
    assert_eq!(
        names(&visible(
            &rows,
            &filter("", Some(DriverState::Loaded), None)
        )),
        ["iwlwifi", "mt76_usb", "r8169"]
    );
    assert_eq!(
        names(&visible(
            &rows,
            &filter("", Some(DriverState::Loaded), Some(Category::Wireless))
        )),
        ["iwlwifi", "mt76_usb"]
    );
    assert_eq!(
        names(&visible(&rows, &filter(" MT76-usb ", None, None))),
        ["mt76_usb"]
    );
    assert_eq!(
        names(&visible(
            &rows,
            &filter("", Some(DriverState::Blacklisted), None)
        )),
        ["b43"]
    );
    assert_eq!(
        names(&visible(&rows, &filter("no-such-driver", None, None))),
        Vec::<String>::new()
    );
}

#[test]
fn a_blacklist_click_queues_and_a_second_takes_it_back() {
    let facts = facts(Vec::new());
    let state = snapshot(&[]);
    let mut changes = ChangeSet::default();

    set_blacklisted(&module("mt76-usb"), true, &facts, &mut changes, &state).unwrap();

    assert_eq!(
        changes.wish::<ModuleBlacklist>(&module("mt76_usb")),
        Some(&Toggle::On)
    );
    assert_eq!(
        row(&driver_rows(&facts, &changes), "mt76_usb").queued,
        Some(Toggle::On)
    );

    set_blacklisted(&module("mt76_usb"), false, &facts, &mut changes, &state).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn an_own_entry_can_be_queued_for_removal() {
    let facts = facts(vec![entry("wl", OWN_FILE), entry("wl", BROADCOM_FILE)]);
    let state = snapshot(&["wl"]);
    let mut changes = ChangeSet::default();

    set_blacklisted(&module("wl"), false, &facts, &mut changes, &state).unwrap();

    assert_eq!(
        changes.wish::<ModuleBlacklist>(&module("wl")),
        Some(&Toggle::Off)
    );
}

#[test]
fn a_blacklist_of_another_file_is_not_changed_here() {
    let facts = facts(vec![entry("b43", BROADCOM_FILE)]);
    let state = snapshot(&[]);
    let mut changes = ChangeSet::default();

    assert_eq!(
        set_blacklisted(&module("b43"), false, &facts, &mut changes, &state),
        Err(Refusal::OtherFile)
    );
    assert_eq!(
        set_blacklisted(&module("b43"), true, &facts, &mut changes, &state),
        Err(Refusal::OtherFile)
    );
    assert!(changes.is_empty());
}

#[test]
fn load_and_unload_are_root_plans_that_run_at_once() {
    let load = plan_load(&module("mt76-usb"));
    let unload = plan_unload(&module("r8169"));
    let middle = |plan: &toolbox_plan::Plan| -> Vec<String> {
        commands(plan)
            .into_iter()
            .filter(|line| line.starts_with("modprobe") && !line.contains("cfg80211"))
            .collect()
    };

    assert_eq!(middle(&load), ["modprobe -- mt76_usb"]);
    assert_eq!(middle(&unload), ["modprobe -r -- r8169"]);
    assert!(
        load.steps()
            .iter()
            .chain(unload.steps())
            .all(|step| matches!(step.action, PlannedAction::Root(_)))
    );
}
