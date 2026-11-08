use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::fill_under;

use super::KernelsPage;
use super::fixtures::{dkms, system};
use super::view::{Dkms, DkmsReading};
use crate::command::{Command, KernelEdit};
use crate::testing::{PageFixture, assert_rows_wait_while_busy};

#[test]
fn before_the_first_read_placeholder_rows_wait() {
    let fixture = PageFixture {
        probed: false,
        ..PageFixture::default()
    };

    let mut harness = fixture.harness(KernelsPage);

    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the kernels…")
            .is_some()
    );
}

#[test]
fn without_dkms_the_page_says_there_is_nothing_to_build() {
    let fixture = PageFixture {
        dkms: DkmsReading::Loaded(Arc::new(Dkms::NotInstalled)),
        ..PageFixture::with_snapshot(system())
    };

    let mut harness = fixture.harness(KernelsPage);

    harness.run();

    assert!(
        harness
            .query_by_label("dkms is not installed, so there are no modules to build.")
            .is_some()
    );
}

#[test]
fn while_dkms_is_asked_an_indicator_says_so() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage);

    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Asking dkms for its modules…")
            .is_some()
    );
}

#[test]
fn while_a_plan_runs_the_kernels_wait_and_a_banner_says_why() {
    assert_rows_wait_while_busy(|| PageFixture::with_snapshot(system()), || KernelsPage);
}

#[test]
fn space_on_a_focused_kernel_flips_its_switch() {
    let mut harness = PageFixture::with_snapshot(system()).harness(KernelsPage);
    harness.run();
    harness
        .get_by_role_and_label(Role::ListItem, "linux-lts")
        .focus();
    harness.run();

    harness.key_press(egui::Key::Space);
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(KernelEdit::Toggle {
            kernel: "linux-lts".parse().unwrap(),
            on: false,
        })]
    );
}

#[test]
fn set_as_default_sits_at_the_right_beside_the_switch() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage);
    harness.run();

    let default = harness
        .get_by_label("Set linux-lts as the default boot entry")
        .rect();

    let switch = harness
        .get_by_role_and_label(Role::CheckBox, "linux-lts")
        .rect();

    assert!(
        default.right() <= switch.left() && switch.left() - default.right() < 24.0,
        "the button ends at {} and the switch starts at {}",
        default.right(),
        switch.left(),
    );
    assert!(
        default.top() < switch.bottom() && default.bottom() > switch.top(),
        "the button is not on the switch's line",
    );
}

#[test]
fn the_switches_line_up_whatever_sits_beside_them() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage);
    harness.run();

    let right = |name: &str| {
        harness
            .get_by_role_and_label(Role::CheckBox, name)
            .rect()
            .right()
    };

    assert!(
        (right("linux-lts") - right("linux-zen")).abs() < 1.0,
        "linux-lts ends at {} and linux-zen at {}",
        right("linux-lts"),
        right("linux-zen"),
    );
}

#[test]
fn the_first_banner_follows_the_header_without_an_extra_gap() {
    let fixture = PageFixture {
        dkms: dkms(),
        ..PageFixture::with_snapshot(system())
    };

    let mut harness = fixture.harness(KernelsPage);
    harness.run();

    let running = harness.get_by_label_contains("Running kernel:").rect();
    let first = harness.get_by_label_contains("lacks its headers").rect();

    // The band reaches a card gap under the header and the body starts a
    // header gap under the band, about 53 px with the banner's padding. A
    // second gap would make it more than 70.
    let gap = first.top() - running.bottom();
    assert!(
        gap < 60.0,
        "{gap} px from the header to the first banner's words"
    );
}

#[test]
fn running_is_a_status_dot_like_the_other_row_states() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage);
    harness.run();

    let running = harness.get_by_label("Running").rect();
    let name = harness.get_by_role_and_label(Role::Label, "linux").rect();
    let fill = fill_under(&harness, running.center());
    let row = fill_under(&harness, name.center());

    assert_eq!(fill, row, "Running sits on its own pill");
}

#[test]
fn the_headers_tag_and_its_button_share_one_centre_line() {
    let fixture = PageFixture::with_snapshot(system());
    let mut harness = fixture.harness(KernelsPage);
    harness.run();

    let tag = harness.get_by_label("Headers missing").rect();
    let button = harness
        .get_by_role_and_label(Role::Button, "Queue headers")
        .rect();

    assert!(
        (tag.center().y - button.center().y).abs() < 0.5,
        "tag at {tag:?}, button at {button:?}"
    );
}
