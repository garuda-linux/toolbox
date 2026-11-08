use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, RepairArgs};
use toolbox_cli::confirm::Prompt;
use toolbox_core::features::bootrepair::{
    Encryption, Firmware, RepairFacts, SAMPLE_LISTING, parse_lsblk,
};

use super::{pick_target, repair};

struct NoTerminal;

impl Prompt for NoTerminal {
    fn is_terminal(&self) -> bool {
        false
    }

    fn ask(&mut self, _question: &str) -> Option<String> {
        None
    }
}

fn facts() -> RepairFacts {
    RepairFacts {
        firmware: Firmware::Uefi,
        missing: Vec::new(),
        inventory: parse_lsblk(SAMPLE_LISTING).unwrap(),
    }
}

fn args(system: &str) -> RepairArgs {
    RepairArgs {
        system: system.to_owned(),
        passphrase_stdin: false,
        boot_repair: true,
        confirm: Confirm::default(),
    }
}

#[test]
fn an_unknown_system_is_not_available() {
    let refused = repair(&facts(), &args("/dev/nope"), None, &mut NoTerminal).unwrap_err();
    assert_eq!(refused.exit(), Exit::Unavailable);
}

#[test]
fn a_locked_system_without_its_passphrase_is_refused() {
    let facts = facts();
    let locked = facts
        .inventory
        .systems
        .iter()
        .find(|system| system.encryption == Encryption::Locked)
        .expect("the sample has a locked system");

    let refused = repair(&facts, &args(locked.device()), None, &mut NoTerminal).unwrap_err();
    assert!(
        refused.message().contains("no passphrase"),
        "{}",
        refused.message()
    );
}

#[test]
fn an_open_system_gets_a_regenerate_plan() {
    let facts = facts();
    let open = facts
        .inventory
        .systems
        .iter()
        .find(|system| system.encryption == Encryption::None && !system.in_use)
        .expect("the sample has an open system");

    let plan = repair(&facts, &args(open.device()), None, &mut NoTerminal).unwrap();
    assert_ne!(plan.steps().len(), 0);
}

#[test]
fn a_target_that_is_not_listed_is_refused() {
    let refused = pick_target(&facts(), "/dev/sda2", Some("/dev/nope")).unwrap_err();
    assert_eq!(refused.exit(), Exit::Unavailable);
}

#[test]
fn the_systems_list_counts_the_ones_in_use() {
    use super::{SystemRow, system_listing};

    let row = |device: &str, in_use| SystemRow {
        device: device.to_owned(),
        label: "Garuda Linux".to_owned(),
        encryption: "none",
        in_use,
    };

    let rows = [row("/dev/sda2", true), row("/dev/sdb2", false)];
    let listing = system_listing(&rows);
    assert_eq!(listing.summary.as_deref(), Some("2, 1 in use"));
    assert_eq!(listing.head, ["DEVICE", "SYSTEM", "STATE"]);
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox bootrepair regenerate <device>")
    );

    assert_eq!(system_listing(&[row("/dev/sda2", true)]).hint, None);
}

#[test]
fn a_refusal_names_the_show_command() {
    let refused = repair(&facts(), &args("/dev/nope"), None, &mut NoTerminal).unwrap_err();
    assert!(
        refused.message().contains("bootrepair show"),
        "{}",
        refused.message()
    );
}
