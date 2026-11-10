//! Captured `lsblk` listings and the edits that turn them into cases no
//! capture shows. Every edit is named where it is made.

use serde_json::Value;
/// The laptop and its edit are the dry run's sample too.
pub use toolbox_state::bootrepair::sample::{BARE_METAL, laptop_from_the_live_iso};

/// A Garuda install in QEMU, booted from its own disk.
pub const INSTALLED: &str = include_str!("../fixtures/installed.json");
/// The Lima VM with a second disk that is LUKS without a partition
/// table, LVM inside and `vg0-root` mounted.
pub const LVM_ON_LUKS: &str = toolbox_state::testing::features::bootrepair::LIMA_LVM_ON_LUKS_JSON;
/// The Garuda live ISO in QEMU with an empty disk.
pub const LIVE: &str = toolbox_state::testing::bootrepair::LIVE_JSON;

/// `vg0-root` and `vg0-swap` below `vdb` of the LVM listing.
const VG0_ROOT: &str = "/blockdevices/2/children/0/children/0";
const VG0_SWAP: &str = "/blockdevices/2/children/0/children/1";

fn edited(listing: &str, edit: impl FnOnce(&mut Value)) -> String {
    let mut value: Value = serde_json::from_str(listing).unwrap();

    edit(&mut value);
    value.to_string()
}

fn at<'value>(value: &'value mut Value, pointer: &str) -> &'value mut Value {
    value
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("{pointer} is in the listing"))
}

/// [`LVM_ON_LUKS`] edited: `vg0-root` unmounted and `vg0-swap` swapped
/// on, so only swap is active on `vdb`.
pub fn lvm_with_only_swap() -> String {
    edited(LVM_ON_LUKS, |value| {
        *at(value, &format!("{VG0_ROOT}/mountpoints")) = Value::Array(Vec::new());
        *at(value, &format!("{VG0_SWAP}/mountpoints")) = serde_json::json!(["[SWAP]"]);
    })
}
