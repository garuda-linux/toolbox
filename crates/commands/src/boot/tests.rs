use toolbox_cli::Exit;
use toolbox_cli::args::{BootKey, BootSet, Confirm, DkmsBuild};
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentGrub, InstalledPackages};
use toolbox_plan::{GrubMenu, GrubState};
use toolbox_system::Host;

use super::{build, default_kernel, set};

const DEFAULTS: &str = "\
GRUB_DEFAULT=gnulinux-simple-1
GRUB_TIMEOUT=5
GRUB_CMDLINE_LINUX_DEFAULT=\"quiet loglevel=3\"
";

const GRUB_CFG: &str = "\
menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {
}
submenu 'Advanced options' $menuentry_id_option 'gnulinux-advanced-1' {
\tmenuentry 'Garuda Linux, with Linux linux-lts' $menuentry_id_option 'gnulinux-linux-lts-advanced-1' {
\t}
}
";

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
    let menu = GrubMenu::from_config(GRUB_CFG);
    let plan = default_kernel(&"linux-lts".parse().unwrap(), &menu, &grub()).unwrap();
    let text = format!("{:?}", plan.steps());
    assert!(text.contains("gnulinux-linux-lts-advanced-1"), "{text}");

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

    let rows = entries(Some(&GrubMenu::from_config(GRUB_CFG)));
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
