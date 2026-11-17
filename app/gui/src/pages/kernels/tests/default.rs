//! The default kernel of the boot menu: its row says Default and offers no
//! Set as default, the others keep the link.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::CurrentGrub;
use toolbox_core::features::kernels::RunningKernel;
use toolbox_core::testing::features::kernels::release;
use toolbox_plan::testing::grub::{LIMA_DEFAULT_GRUB, LIMA_GRUB_CFG};
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState};
use toolbox_services::boot_menu::Menu;
use toolbox_system::MenuRead;

use crate::boot::Boot;
use crate::pages::kernels::KernelsPage;
use crate::pages::kernels::fixtures::{package, system};
use crate::testing::PageFixture;

/// linux-lts's entry in the Lima capture, a submenu entry of GRUB.
const LTS_ENTRY: &str = "gnulinux-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6>gnulinux-linux-lts-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6";

/// `snapshot` with GRUB's default set to `value` and its menu read.
fn with_grub_default_on(mut snapshot: Snapshot, value: &str) -> PageFixture {
    let text = LIMA_DEFAULT_GRUB.replacen("GRUB_DEFAULT=0", &format!("GRUB_DEFAULT='{value}'"), 1);
    snapshot.insert(CurrentGrub(GrubState::read(&text)));
    let menu = GrubMenu::for_platform(LIMA_GRUB_CFG, GrubPlatform::Efi);

    let mut fixture = PageFixture::with_snapshot(snapshot);
    fixture.services.boot = Boot::showing(Menu::Read(MenuRead::Known(menu)), None, None);
    fixture
}

#[test]
fn the_default_kernel_says_default_and_offers_no_set_as_default() {
    let mut harness = with_grub_default_on(system(), LTS_ENTRY).harness(KernelsPage::default());
    harness.run();

    let shown = [
        harness.query_by_label("Default").is_some(),
        harness
            .query_by_label("Set linux-lts as the default boot entry")
            .is_some(),
    ];

    assert_eq!(shown, [true, false]);
}

#[test]
fn another_installed_kernel_keeps_its_set_as_default_link() {
    let mut harness = with_grub_default_on(system(), LTS_ENTRY).harness(KernelsPage::default());
    harness.run();

    let link = harness.query_by_role_and_label(Role::Button, "Set linux as the default boot entry");

    assert!(link.is_some());
}

#[test]
fn without_a_read_menu_no_kernel_says_default_and_all_keep_their_link() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());
    harness.run();

    let shown = [
        harness.query_by_label("Default").is_some(),
        harness
            .query_by_label("Set linux-lts as the default boot entry")
            .is_some(),
    ];

    assert_eq!(shown, [false, true]);
}

#[test]
fn a_running_default_kernel_shows_running_and_default_side_by_side() {
    let mut snapshot = system();
    snapshot.insert(RunningKernel {
        release: release("6.12.9-1-lts"),
        package: Some(package("linux-lts")),
    });
    let mut harness = with_grub_default_on(snapshot, LTS_ENTRY).harness(KernelsPage::default());
    harness.run();

    let running = harness.get_by_label("Running").rect();
    let default = harness.get_by_label("Default").rect();

    assert!(
        running.right() <= default.left(),
        "Running ends at {} and Default starts at {}",
        running.right(),
        default.left(),
    );
    assert!((running.center().y - default.center().y).abs() < 2.0);
}
