use serde_json::Value;
use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, RepairArgs};
use toolbox_cli::confirm::Prompt;
use toolbox_core::features::bootrepair::{Encryption, Firmware, RepairFacts, parse_lsblk};

use super::{pick_target, repair};
use crate::change::Refused;

/// A bare-metal laptop: the running system on `nvme0n1` in LUKS, a free second
/// Garuda on `nvme1n1p2`.
const BARE_METAL: &str =
    include_str!("../../../core/src/features/bootrepair/fixtures/bare-metal.json");

struct NoTerminal;

impl Prompt for NoTerminal {
    fn is_terminal(&self) -> bool {
        false
    }

    fn ask(&mut self, _question: &str) -> Option<String> {
        None
    }
}

fn facts_of(listing: &str) -> RepairFacts {
    RepairFacts {
        firmware: Firmware::Uefi,
        missing: Vec::new(),
        inventory: parse_lsblk(listing).unwrap(),
    }
}

fn facts() -> RepairFacts {
    facts_of(BARE_METAL)
}

/// [`BARE_METAL`] edited: the mapped child of the LUKS partition
/// `nvme0n1p1` cut, so it is locked, as on the live ISO.
fn locked_laptop() -> RepairFacts {
    let mut listing: Value = serde_json::from_str(BARE_METAL).unwrap();

    listing
        .pointer_mut("/blockdevices/3/children/0")
        .and_then(Value::as_object_mut)
        .unwrap()
        .remove("children");

    facts_of(&listing.to_string())
}

fn args(system: &str) -> RepairArgs {
    RepairArgs {
        system: system.to_owned(),
        passphrase_stdin: false,
        boot_repair: true,
        trust_disk: false,
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
    let facts = locked_laptop();
    let locked = facts.inventory.system("/dev/nvme0n1p1").unwrap();

    assert_eq!(locked.encryption, Encryption::Locked);

    let refused = repair(&facts, &args(locked.device()), None, &mut NoTerminal).unwrap_err();
    assert!(
        refused.message().contains("no passphrase"),
        "{}",
        refused.message()
    );
}

#[test]
fn an_open_system_gets_a_regenerate_plan() {
    let plan = repair(&facts(), &args("/dev/nvme1n1p2"), None, &mut NoTerminal).unwrap();
    assert_ne!(plan.steps().len(), 0);
}

#[test]
fn a_target_that_is_not_listed_is_refused() {
    let refused = pick_target(&facts(), "/dev/nvme1n1p2", Some("/dev/nope")).unwrap_err();
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

#[test]
fn a_system_the_running_system_uses_says_why_in_english() {
    let refused = repair(&facts(), &args("/dev/mapper/garuda"), None, &mut NoTerminal).unwrap_err();
    assert_eq!(refused.exit(), Exit::PageRule);
    assert_eq!(
        refused.message(),
        "the running system uses this disk, so nothing is written to it"
    );
}

/// Outside the live ISO the chosen disk's own programs run as root, so the
/// command line asks for the user's word first.
#[test]
fn an_installed_system_needs_trust_in_the_disk() {
    let trusted = RepairArgs {
        trust_disk: true,
        ..args("/dev/sda2")
    };

    assert_eq!(super::offered(true, &args("/dev/sda2")).ok(), Some(()));
    assert_eq!(super::offered(false, &trusted).ok(), Some(()));

    let refused = super::offered(false, &args("/dev/sda2")).unwrap_err();
    let Refused::Rule(words) = refused else {
        panic!("{refused:?}");
    };

    assert!(words.contains("--trust-disk"), "{words}");
}
