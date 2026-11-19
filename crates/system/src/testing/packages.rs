/// The local database captured in the Lima VM, a folder to copy from.
pub const ALPM_LOCAL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/packages/fixtures/alpm/var/lib/pacman/local"
);

/// `/etc/pacman.conf` of the Lima VM.
pub const PACMAN_CONF: &str = include_str!("../packages/pacman_conf/fixtures/pacman.conf");
