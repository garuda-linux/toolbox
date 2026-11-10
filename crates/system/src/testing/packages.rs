/// The local database captured in the Lima VM, a folder to copy from.
pub const ALPM_LOCAL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/packages/fixtures/alpm/var/lib/pacman/local"
);
