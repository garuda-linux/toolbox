use toolbox_plan::KernelModule;
use toolbox_state::network::ModuleFacts;
use toolbox_state::parts::OwnBlacklist;

use super::fakes::FakePackages;
use super::{host, installed, probe_snapshot, sources};

/// The app's own blacklist is a part of the snapshot, so the page lists
/// the blacklisted module once the probe has read it.
#[test]
fn the_own_module_blacklist_is_a_part_of_the_snapshot() {
    let (snapshot, issues) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));
    let wl = KernelModule::new("wl").unwrap();

    assert_eq!(issues, []);
    assert_eq!(
        snapshot.part::<OwnBlacklist>(),
        Some(&OwnBlacklist(std::iter::once(wl).collect()))
    );
}

/// The kernel's modules come with the snapshot too, so the guard of a
/// restored blacklist change can check it before the page is opened.
#[test]
fn the_kernel_modules_are_a_part_of_the_snapshot() {
    let (snapshot, _issues) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));

    assert_eq!(
        snapshot.part::<ModuleFacts>(),
        Some(&ModuleFacts::default())
    );
}
