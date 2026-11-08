use toolbox_catalog::ToggleCatalog;

use super::snapshot;
use crate::domains::{EnabledUserUnits, Packages, SystemUnits, UserUnits};
use crate::features::toggles::summary::summary;
use crate::features::toggles::{state, toggle};
use crate::testing::{package, unit};
use crate::{ChangeSet, Snapshot, Toggle};

const SETUP: &str = r#"
[[section]]
id = "office"
title = "Office"
icon = "x"

[[section.entry]]
id = "fresh"
title = "LibreOffice Fresh"
description = "F"
check = { package = "libreoffice-fresh" }
also = [{ package = "libmythes" }]
conflicts = ["still"]

[[section.entry]]
id = "still"
title = "LibreOffice Still"
description = "S"
check = { package = "libreoffice-still" }
also = [{ package = "libmythes" }]
conflicts = ["fresh"]

[[section]]
id = "virtualization"
title = "Virtualization"
icon = "x"

[[section.entry]]
id = "boxes"
title = "GNOME Boxes"
description = "B"
check = { package = "gnome-boxes" }
also = [{ service = "libvirtd.service" }]

[[section.entry]]
id = "virt-manager"
title = "Virtual Machine Manager"
description = "V"
check = { package = "virt-manager" }
also = [{ service = "libvirtd.service" }]

[[section.entry]]
id = "podman"
title = "Podman"
description = "P"
check = { package = "podman-docker" }
also = [{ package = "crun" }, { service = "podman.socket" }, { user-service = "podman.socket" }]

[[section.entry]]
id = "virtualbox"
title = "VirtualBox"
description = "V"
check = { package = "virtualbox" }

[[section.entry]]
id = "genymotion"
title = "Genymotion"
description = "G"
check = { package = "genymotion" }
also = [{ package = "virtualbox" }]
"#;

fn catalog() -> ToggleCatalog {
    ToggleCatalog::parse(SETUP, "test.toml").unwrap()
}

fn click(id: &str, on: bool, system: &Snapshot, changes: &mut ChangeSet) {
    toggle(&catalog(), id, Toggle::from_on(on), system, changes)
        .unwrap_or_else(|refusal| panic!("{id}: {refusal:?}"));
}

fn wish(name: &str, changes: &ChangeSet) -> Option<Toggle> {
    changes.wish::<Packages>(&package(name)).copied()
}

fn unit_wish(name: &str, changes: &ChangeSet) -> Option<Toggle> {
    changes.wish::<SystemUnits>(&unit(name)).copied()
}

fn is_on(id: &str, system: &Snapshot, changes: &ChangeSet) -> bool {
    let catalog = catalog();

    state(&catalog, catalog.entry(id).unwrap(), system, changes).shown_on()
}

#[test]
fn turning_on_enables_the_units_in_both_scopes() {
    let mut system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    system.insert(EnabledUserUnits::default());
    click("podman", true, &system, &mut changes);

    assert_eq!(wish("podman-docker", &changes), Some(Toggle::On));
    assert_eq!(wish("crun", &changes), Some(Toggle::On));
    assert_eq!(unit_wish("podman.socket", &changes), Some(Toggle::On));
    assert_eq!(
        changes.wish::<UserUnits>(&unit("podman.socket")),
        Some(&Toggle::On)
    );

    click("podman", false, &system, &mut changes);

    assert!(changes.is_empty());
}

#[test]
fn turning_off_an_installed_entry_disables_its_enabled_units() {
    let system = snapshot(&["gnome-boxes"], &["libvirtd.service"], &[]);
    let mut changes = ChangeSet::default();

    click("boxes", false, &system, &mut changes);

    assert_eq!(wish("gnome-boxes", &changes), Some(Toggle::Off));
    assert_eq!(unit_wish("libvirtd.service", &changes), Some(Toggle::Off));
}

#[test]
fn a_unit_another_installed_entry_needs_stays_enabled() {
    let system = snapshot(&["gnome-boxes", "virt-manager"], &["libvirtd.service"], &[]);
    let mut changes = ChangeSet::default();

    click("boxes", false, &system, &mut changes);

    assert_eq!(wish("gnome-boxes", &changes), Some(Toggle::Off));
    assert_eq!(unit_wish("libvirtd.service", &changes), None);

    click("virt-manager", false, &system, &mut changes);

    assert_eq!(unit_wish("libvirtd.service", &changes), Some(Toggle::Off));

    // Back on: nobody else needs the unit, so its queued "off" goes too.
    click("boxes", true, &system, &mut changes);

    assert_eq!(wish("gnome-boxes", &changes), None);
    assert_eq!(unit_wish("libvirtd.service", &changes), None);
    assert_eq!(wish("virt-manager", &changes), Some(Toggle::Off));
}

#[test]
fn a_shared_package_stays_queued_while_another_queued_entry_needs_it() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    click("boxes", true, &system, &mut changes);
    click("virt-manager", true, &system, &mut changes);
    click("boxes", false, &system, &mut changes);

    assert_eq!(wish("gnome-boxes", &changes), None);
    assert_eq!(unit_wish("libvirtd.service", &changes), Some(Toggle::On));

    click("virt-manager", false, &system, &mut changes);

    assert!(changes.is_empty());
}

#[test]
fn an_also_package_that_is_another_entrys_own_package_stays() {
    let system = snapshot(&["genymotion", "virtualbox"], &[], &[]);
    let mut changes = ChangeSet::default();

    click("genymotion", false, &system, &mut changes);

    assert_eq!(wish("genymotion", &changes), Some(Toggle::Off));
    assert_eq!(wish("virtualbox", &changes), None);
    assert!(is_on("virtualbox", &system, &changes));
}

#[test]
fn turning_on_takes_a_queued_conflict_back() {
    let system = snapshot(&[], &[], &[]);
    let mut changes = ChangeSet::default();

    click("still", true, &system, &mut changes);
    click("fresh", true, &system, &mut changes);

    assert_eq!(wish("libreoffice-still", &changes), None);
    assert_eq!(wish("libreoffice-fresh", &changes), Some(Toggle::On));
    assert_eq!(wish("libmythes", &changes), Some(Toggle::On));
    assert!(!is_on("still", &system, &changes));
}

#[test]
fn turning_on_queues_the_removal_of_an_installed_conflict() {
    let system = snapshot(&["libreoffice-still", "libmythes"], &[], &[]);
    let mut changes = ChangeSet::default();

    click("fresh", true, &system, &mut changes);

    assert_eq!(wish("libreoffice-still", &changes), Some(Toggle::Off));
    assert_eq!(wish("libreoffice-fresh", &changes), Some(Toggle::On));
    assert_eq!(wish("libmythes", &changes), None);

    // Taking the choice back leaves the installed branch queued off: the
    // user unticked nothing, so the page shows both and they decide.
    click("fresh", false, &system, &mut changes);

    assert_eq!(wish("libreoffice-fresh", &changes), None);
    assert_eq!(wish("libreoffice-still", &changes), Some(Toggle::Off));
}

#[test]
fn turning_off_touches_no_conflict() {
    let system = snapshot(&["libreoffice-still", "libreoffice-fresh"], &[], &[]);
    let mut changes = ChangeSet::default();

    click("fresh", false, &system, &mut changes);

    assert_eq!(wish("libreoffice-still", &changes), None);
}

#[test]
fn the_summary_lists_what_the_catalog_queued() {
    let system = snapshot(&["virt-manager"], &["libvirtd.service"], &[]);
    let mut changes = ChangeSet::default();
    let catalog = catalog();

    assert!(summary(&catalog, &system, &changes).is_empty());

    click("podman", true, &system, &mut changes);
    click("virt-manager", false, &system, &mut changes);
    changes.want::<Packages>(package("htop"), Toggle::On, &system);

    let queued = summary(&catalog, &system, &changes);
    let ids = |entries: &[&toolbox_catalog::ToggleEntry]| -> Vec<String> {
        entries.iter().map(|entry| entry.id().to_owned()).collect()
    };

    assert_eq!(ids(&queued.install), ["podman"]);
    assert_eq!(ids(&queued.remove), ["virt-manager"]);
    assert_eq!(queued.units.len(), 1);
    assert_eq!(queued.user_units.len(), 1);
    assert!(!queued.is_empty());
}
