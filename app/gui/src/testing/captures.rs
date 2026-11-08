//! Captures that more than one test module of the app reads.

/// The `grub.cfg` of a Garuda Mokka install, UUIDs replaced by `x`.
pub const GRUB_CFG: &str =
    include_str!("../../../../crates/plan/src/grub/fixtures/installed/grub.cfg");
