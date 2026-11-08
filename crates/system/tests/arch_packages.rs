//! libalpm against the real databases of the Arch test image
//! (`ci/arch/Containerfile`), found through the system's own
//! `pacman.conf`. It only runs there: `TOOLBOX_IT_ARCH=1`, root and
//! `/etc/arch-release`. Elsewhere it returns at once, so no unit test
//! depends on the machine it runs on. Run it with
//! `scripts/arch-it.sh -p toolbox-system --test arch_packages`.

mod common;

use std::path::Path;

use toolbox_system::{Libalpm, PackageSource as _, ProbeError};

/// What a system without databases answers is tested with a made-up one
/// (`crates/system/src/packages/tests.rs`).
#[test]
fn the_real_system_answers() {
    if !common::in_test_image() {
        return;
    }

    assert!(Path::new("/etc/pacman.conf").is_file());
    assert!(Path::new("/var/lib/pacman/local").is_dir());
    assert_ne!(Libalpm::system().installed(), Err(ProbeError::Unavailable));
}
