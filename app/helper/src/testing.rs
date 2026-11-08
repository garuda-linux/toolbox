//! Captures that more than one test module of the helper reads.

/// The Lima VM's `/etc/shadow`, hashes replaced by `x`.
pub const SHADOW: &str = include_str!("../../../crates/plan/src/account/fixtures/shadow");
