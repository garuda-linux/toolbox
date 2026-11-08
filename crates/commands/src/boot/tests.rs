use toolbox_cli::Exit;
use toolbox_cli::args::{BootKey, BootSet, Confirm, DkmsBuild};
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentGrub, InstalledPackages};
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState};
use toolbox_system::Host;

use super::{build, default_kernel, set};

/// `/etc/default/grub` and `grub.cfg` of a Garuda Mokka install, UUIDs
/// of `grub.cfg` replaced by `x`.
const DEFAULTS: &str = include_str!("../../../plan/src/grub/fixtures/installed/default-grub");
const GRUB_CFG: &str = include_str!("../../../plan/src/grub/fixtures/installed/grub.cfg");

fn grub() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Host::default());
    snapshot.insert(CurrentGrub(GrubState::read(DEFAULTS)));
    snapshot.insert(InstalledPackages::default());
    snapshot
}

fn setting(key: BootKey, value: &str) -> BootSet {
    BootSet {
        key,
        value: value.to_owned(),
        confirm: Confirm::default(),
    }
}

#[test]
fn a_new_timeout_is_one_grub_write() {
    let plan = set(&setting(BootKey::Timeout, "3"), &grub()).unwrap();
    let text = format!("{:?}", plan.steps());
    assert!(text.contains("Timeout"), "{text}");
}

#[test]
fn a_switch_takes_on_or_off_only() {
    let refused = set(&setting(BootKey::Quiet, "maybe"), &grub()).unwrap_err();
    assert!(refused.message().contains("neither on nor off"));

    set(&setting(BootKey::Quiet, "off"), &grub()).unwrap();
}

#[test]
fn a_timeout_outside_the_page_s_range_is_refused_with_its_rule() {
    let refused = set(&setting(BootKey::Timeout, "9000"), &grub()).unwrap_err();
    assert_eq!(refused.exit(), Exit::PageRule);
}

#[test]
fn without_grub_every_setting_is_refused() {
    set(&setting(BootKey::Timeout, "3"), &Snapshot::default()).unwrap_err();
}

#[test]
fn a_kernel_s_entry_becomes_the_default() {
    let menu = GrubMenu::for_platform(GRUB_CFG, GrubPlatform::Efi);
    let plan = default_kernel(&"linux-garuda".parse().unwrap(), &menu, &grub()).unwrap();
    let text = format!("{:?}", plan.steps());
    assert!(text.contains("gnulinux-linux-garuda-advanced-"), "{text}");

    let refused = default_kernel(&"linux-zen".parse().unwrap(), &menu, &grub()).unwrap_err();
    assert_eq!(refused.exit(), Exit::Unavailable);
}

#[test]
fn without_pacman_s_databases_no_build_is_planned() {
    let args = DkmsBuild {
        kernel: None,
        confirm: Confirm::default(),
    };

    build(&args, &[], &Snapshot::default()).unwrap_err();
}

#[test]
fn dkms_modules_are_marked_by_their_build() {
    use toolbox_cli::output::Mark;

    use super::{ModuleRow, module_listing};

    let row = |module: &str, state| ModuleRow {
        module: module.to_owned(),
        version: "1.0".to_owned(),
        kernel: "6.1.0".to_owned(),
        state,
    };

    let rows = [
        row("nvidia", "installed"),
        row("zfs", "built"),
        row("vbox", "added"),
        row("v4l2", "broken"),
    ];

    let listing = module_listing(&rows);
    let marks: Vec<Mark> = listing.rows.iter().map(|shown| shown.mark).collect();

    assert_eq!(marks, [Mark::On, Mark::On, Mark::Off, Mark::Problem]);
    assert_eq!(listing.head, Vec::<&str>::new());
    assert_eq!(listing.summary.as_deref(), Some("4, 2 built, 1 broken"));
    assert_eq!(listing.hint.as_deref(), Some("garuda-toolbox dkms build"));

    let built = module_listing(&rows[..2]);
    assert_eq!(built.summary.as_deref(), Some("2, 2 built"));
    assert_eq!(built.hint, None);
}

#[test]
fn boot_entries_name_the_command_that_takes_them() {
    use toolbox_core::features::boot::view::entries;

    use super::entry_listing;

    let rows = entries(Some(&GrubMenu::for_platform(GRUB_CFG, GrubPlatform::Efi)));
    let listing = entry_listing(&rows);

    assert_eq!(listing.summary.as_deref(), Some(&*rows.len().to_string()));
    assert_eq!(listing.head, ["ENTRY", "TITLE"]);
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox boot set default <entry>")
    );
}

#[test]
fn a_dkms_row_as_json_names_its_state() {
    use super::ModuleRow;

    let row = serde_json::to_value(ModuleRow {
        module: "nvidia".to_owned(),
        version: "1.0".to_owned(),
        kernel: "6.1.0".to_owned(),
        state: "installed",
    })
    .unwrap();

    assert_eq!(row["id"], "nvidia");
    assert_eq!(row["state"], "installed");
    assert!(row.get("status").is_none(), "{row}");
}

/// The value `boot set default` takes, named like every other list's id.
#[test]
fn a_boot_entry_as_json_has_its_id() {
    use toolbox_core::features::boot::view::EntryRow;

    let row = serde_json::to_value(EntryRow {
        entry: "gnulinux-simple-1".to_owned(),
        title: "Garuda Linux".to_owned(),
        parent: String::new(),
    })
    .unwrap();

    assert_eq!(row["id"], "gnulinux-simple-1");
    assert!(row.get("entry").is_none(), "{row}");
}
