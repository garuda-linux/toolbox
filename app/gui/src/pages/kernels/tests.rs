mod cache;
mod default;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::kernels::parse_status;
use toolbox_gui_kit::testing::{fill_under, press_until, tab_into};
use toolbox_services::dkms::Dkms;

use super::KernelsPage;
use super::fixtures::{dkms, system};
use crate::command::{Command, KernelCommand, KernelEdit};
use crate::kernels::{DkmsReading, Kernels};
use crate::page_service::Services;
use crate::testing::{PLAN_RUNNING, PageFixture, failure_read_again_sends};

fn with_dkms(reading: DkmsReading) -> PageFixture {
    PageFixture {
        services: Services {
            kernels: Kernels::with_dkms(reading),
            ..Services::default()
        },
        ..PageFixture::with_snapshot(system())
    }
}

#[test]
fn before_the_first_read_placeholder_rows_wait() {
    let fixture = PageFixture {
        probed: false,
        ..PageFixture::default()
    };

    let mut harness = fixture.harness(KernelsPage::default());

    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the kernels…")
            .is_some()
    );
}

#[test]
fn unread_package_databases_offer_to_read_again() {
    assert_eq!(
        failure_read_again_sends(PageFixture::default(), KernelsPage::default()),
        [Command::from(KernelCommand::Refresh)]
    );
}

#[test]
fn without_dkms_the_page_says_there_is_nothing_to_build() {
    let fixture = with_dkms(DkmsReading::Loaded(Arc::new(Dkms::NotInstalled)));

    let mut harness = fixture.harness(KernelsPage::default());

    harness.run();

    assert!(
        harness
            .query_by_label("dkms is not installed, so there are no modules to build.")
            .is_some()
    );
}

#[test]
fn while_dkms_is_asked_an_indicator_says_so() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());

    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Asking dkms for its modules…")
            .is_some()
    );
}

#[test]
fn while_a_plan_runs_the_kernels_wait_and_a_banner_says_why() {
    for busy in [true, false] {
        let fixture = PageFixture {
            busy,
            ..PageFixture::with_snapshot(system())
        };

        let mut harness = fixture.harness(KernelsPage::default());
        harness.run();

        let install = harness.get_by_role_and_label(Role::Button, "Install linux-zen");
        assert_eq!(install.accesskit_node().is_disabled(), busy, "busy: {busy}");
        assert_eq!(
            harness.query_by_label(PLAN_RUNNING).is_some(),
            busy,
            "the banner"
        );
    }
}

#[test]
fn space_on_a_focused_kernel_asks_for_its_change() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());
    harness.run();
    assert!(tab_into(&mut harness, Role::ListItem));
    assert!(press_until(
        &mut harness,
        egui::Key::ArrowDown,
        Role::ListItem,
        "linux-lts",
    ));

    harness.key_press(egui::Key::Space);
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(KernelEdit::Toggle {
            kernel: "linux-lts".parse().unwrap(),
            on: toolbox_core::Toggle::Off,
        })]
    );
}

#[test]
fn set_as_default_sits_at_the_right_beside_the_button() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();

    let default = harness
        .get_by_label("Set linux-lts as the default boot entry")
        .rect();

    let remove = harness
        .get_by_role_and_label(Role::Button, "Remove linux-lts")
        .rect();

    assert!(
        default.right() <= remove.left() && remove.left() - default.right() < 24.0,
        "the default button ends at {} and Remove starts at {}",
        default.right(),
        remove.left(),
    );
    assert!(
        default.top() < remove.bottom() && default.bottom() > remove.top(),
        "the default button is not on Remove's line",
    );
}

#[test]
fn the_buttons_line_up_whatever_sits_beside_them() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();

    let rect = |name: &str| harness.get_by_role_and_label(Role::Button, name).rect();
    let lts = rect("Remove linux-lts");
    let zen = rect("Install linux-zen");

    assert!(
        (lts.right() - zen.right()).abs() < 1.0,
        "linux-lts ends at {} and linux-zen at {}",
        lts.right(),
        zen.right(),
    );
    assert!(
        (lts.width() - zen.width()).abs() < 1.0,
        "Remove is {} wide and Install {}",
        lts.width(),
        zen.width(),
    );
}

#[test]
fn the_running_kernel_has_a_resting_remove_button_and_no_row_a_switch() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());
    harness.run();

    let remove = harness.get_by_role_and_label(Role::Button, "Remove linux");
    assert!(remove.accesskit_node().is_disabled());
    assert!(harness.query_by_role(Role::CheckBox).is_none());
}

#[test]
fn a_click_on_the_row_beside_its_button_asks_for_nothing() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());
    harness.run();

    harness
        .get_by_role_and_label(Role::Label, "linux-lts")
        .click();
    harness.run();

    assert_eq!(harness.state().fixture.sent(), []);
}

#[test]
fn the_first_banner_follows_the_header_without_an_extra_gap() {
    let fixture = with_dkms(dkms());

    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();

    let running = harness.get_by_label_contains("Running kernel:").rect();
    let first = harness.get_by_label_contains("lacks its headers").rect();

    let purpose = harness
        .get_by_label(
            "A kernel you install is used after the next restart, chosen in the boot menu.",
        )
        .rect();

    assert!(
        running.top() >= purpose.bottom(),
        "the release is not in the body under the purpose"
    );
    let gap = first.top() - running.bottom();
    assert!(
        gap < 60.0,
        "{gap} px from the release line to the first banner's words"
    );
}

#[test]
fn running_is_a_status_dot_like_the_other_row_states() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();

    let running = harness.get_by_label("Running").rect();
    let name = harness.get_by_role_and_label(Role::Label, "linux").rect();
    let fill = fill_under(&harness, running.center());
    let row = fill_under(&harness, name.center());

    assert_eq!(fill, row, "Running sits on its own pill");
}

#[test]
fn installed_headers_take_no_line_and_missing_ones_do() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage::default());
    harness.run();

    assert!(harness.query_by_label("Headers installed").is_none());
    assert!(harness.query_by_label("Headers missing").is_some());
}

#[test]
fn built_modules_fold_away_and_a_click_opens_them() {
    // The fixture's dkms status without vboxhost, so nothing is left to build.
    let status = "nvidia/550.90.07, 6.14.0-arch3-1, x86_64: installed\n";
    let fixture = with_dkms(DkmsReading::Loaded(Arc::new(Dkms::Modules(parse_status(
        status,
    )))));

    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();
    assert!(
        harness
            .query_by_label("nvidia 550.90.07: installed")
            .is_none()
    );

    harness
        .get_by_role_and_label(Role::Button, "DKMS Modules 1")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_label("nvidia 550.90.07: installed")
            .is_some()
    );
}

#[test]
fn a_kernel_with_a_build_to_do_shows_its_modules() {
    let fixture = with_dkms(dkms());

    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();

    assert!(
        harness
            .query_by_label("vboxhost 7.1.6_OSE: missing")
            .is_some()
    );
}

#[test]
fn the_headers_tag_and_its_button_share_one_centre_line() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage::default());
    harness.run();

    let tag = harness.get_by_label("Headers missing").rect();
    let button = harness
        .get_by_role_and_label(Role::Button, "Queue Headers")
        .rect();

    assert!(
        (tag.center().y - button.center().y).abs() < 0.5,
        "tag at {tag:?}, button at {button:?}"
    );
}

#[test]
fn a_tall_kernel_row_keeps_set_as_default_and_remove_on_its_title_line() {
    let mut harness = with_dkms(dkms()).harness(KernelsPage::default());
    harness.run();

    // The actions centre on the middle of the title's capitals.
    let capitals = toolbox_gui_kit::testing::capitals_top(&harness, "linux")
        .zip(toolbox_gui_kit::testing::baseline(&harness, "linux"))
        .map(|(top, baseline)| f32::midpoint(top, baseline))
        .expect("the title is painted");

    let default = harness
        .get_by_role_and_label(Role::Button, "Set linux as the default boot entry")
        .rect();

    let remove = harness
        .get_by_role_and_label(Role::Button, "Remove linux")
        .rect();

    assert!(
        (default.center().y - capitals).abs() < 1.0 && (remove.center().y - capitals).abs() < 1.0,
        "the title's capitals centre at {capitals} and its actions at {} and {}",
        default.center().y,
        remove.center().y,
    );
}

#[test]
fn a_click_on_a_tall_kernel_row_beside_its_buttons_asks_for_nothing() {
    let mut harness = with_dkms(dkms()).harness(KernelsPage::default());
    harness.run();

    harness.get_by_role_and_label(Role::Label, "linux").click();
    harness.run();

    assert_eq!(harness.state().fixture.sent(), []);
}

#[test]
fn a_click_on_the_middle_of_a_tall_kernel_row_asks_for_nothing() {
    let mut harness = with_dkms(dkms()).harness(KernelsPage::default());
    harness.run();

    harness
        .get_by_role_and_label(Role::ListItem, "linux")
        .click();
    harness.run();

    assert_eq!(harness.state().fixture.sent(), []);
}
