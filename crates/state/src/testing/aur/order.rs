//! `paru -Po --aur <name>` (paru 2.2.1) on the Garuda Lima VM. Both runs
//! printed nothing on stderr.

pub const OCTOPI_GIT_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-octopi-git.txt");
pub const OCTOPI_GIT_CODE: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-octopi-git.code");
pub const MISSING_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-missing.stdout");
pub const MISSING_CODE: &str = include_str!("../../aur/order/fixtures/lima/paru-po-missing.code");
/// `paru -Po firefox`: a queued name that resolves to a repository.
pub const FIREFOX_STDOUT: &str = include_str!("../../aur/order/fixtures/lima/paru-po-firefox.txt");
/// `paru -Po --aur aur-sync-vote`: MAKE lines, then the AUR target.
pub const AUR_SYNC_VOTE_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-aur-sync-vote.txt");
/// `paru -Po --aur nvidia-470xx-utils opencl-nvidia-470xx`: one pkgbase, two packages.
pub const NVIDIA_SPLIT_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-nvidia-470xx-split.txt");
/// `paru -Po --aur cbonsai` with cbonsai 1.4.0 installed and its make
/// dependencies installed: the AUR target alone.
pub const CBONSAI_OUTDATED_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-cbonsai-outdated.txt");
/// The same with the make dependency `scdoc` not installed.
pub const CBONSAI_OUTDATED_MAKE_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-cbonsai-outdated-make.txt");
/// `paru -Po --aur pipes.sh` for an installed package that is current: paru
/// orders its rebuild anyway.
pub const PIPES_SH_CURRENT_STDOUT: &str =
    include_str!("../../aur/order/fixtures/lima/paru-po-pipes-sh-current.txt");
