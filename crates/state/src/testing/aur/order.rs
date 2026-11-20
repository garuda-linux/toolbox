//! `paru -Po --aur <name>` (paru 2.2.1) on the Garuda Lima VM. Both runs
//! printed nothing on stderr.

pub const OCTOPI_GIT_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-octopi-git.txt");
pub const OCTOPI_GIT_CODE: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-octopi-git.code");
pub const MISSING_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-missing.stdout");
pub const MISSING_CODE: &str = include_str!("../../aur/order/fixtures/lima/paru-po-missing.code");
