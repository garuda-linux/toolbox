//! The disks a dry run of a self-test build shows instead of the real
//! ones, so its page neither depends on the machine it runs on nor shows
//! it. Release builds leave this out.

use serde_json::{Map, Value};

/// A bare-metal laptop: the running system on `nvme0n1` in LUKS, a second
/// Garuda on `nvme1n1`, two empty card readers.
pub const BARE_METAL: &str =
    include_str!("../../../core/src/features/bootrepair/fixtures/bare-metal.json");

/// `nvme0n1` of [`BARE_METAL`].
const LAPTOP_DISK: &str = "/blockdevices/3/children";

/// [`BARE_METAL`] as the live ISO would see it.
///
/// Edited: the mount points of the EFI partition `nvme0n1p2` emptied, and
/// the mapped children of the LUKS partitions `nvme0n1p1` and `nvme0n1p3`
/// cut, so both are locked.
///
/// # Panics
///
/// Never for the capture it edits, which its tests read.
#[expect(
    clippy::expect_used,
    reason = "the edited listing is a fixed capture, read by the tests"
)]
#[must_use = "the edited listing is only worth its parse"]
pub fn laptop_from_the_live_iso() -> String {
    let mut value: Value = serde_json::from_str(BARE_METAL).expect("the capture is JSON");

    partition(&mut value, 1).insert("mountpoints".to_owned(), Value::Array(Vec::new()));
    for luks in [0, 2] {
        partition(&mut value, luks).remove("children");
    }

    value.to_string()
}

#[expect(
    clippy::expect_used,
    reason = "the edited listing is a fixed capture, read by the tests"
)]
fn partition(value: &mut Value, index: u8) -> &mut Map<String, Value> {
    value
        .pointer_mut(&format!("{LAPTOP_DISK}/{index}"))
        .and_then(Value::as_object_mut)
        .expect("the laptop's disk has three partitions")
}
