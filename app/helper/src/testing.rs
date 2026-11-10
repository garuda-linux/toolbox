//! Captures that more than one test module of the helper reads.

/// The Lima VM's `/etc/shadow`, hashes replaced by `x`.
pub const SHADOW: &str = toolbox_plan::testing::account::SHADOW;

/// The Lima VM's `/etc/passwd`, its user renamed to `user`.
pub const PASSWD: &str = toolbox_plan::testing::accounts::PASSWD;
