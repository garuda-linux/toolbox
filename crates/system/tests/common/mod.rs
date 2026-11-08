use std::path::Path;

/// The tests run only in the Arch test image: `TOOLBOX_IT_ARCH=1`, root and
/// `/etc/arch-release`. Elsewhere they return at once.
pub fn in_test_image() -> bool {
    std::env::var("TOOLBOX_IT_ARCH").as_deref() == Ok("1")
        && rustix::process::geteuid().is_root()
        && Path::new("/etc/arch-release").exists()
}
