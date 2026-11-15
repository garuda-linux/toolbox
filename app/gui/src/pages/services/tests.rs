mod ask;
mod boot;
mod failure;
mod filters;
mod live;
mod menu;

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::services::{ActiveState, FileState, Shown, UnitInfo};
use toolbox_gui_kit::testing::{press_until, tab_into};
use toolbox_plan::{UnitName, UnitScope};

use super::{ServicesPage, row};
use crate::command::{Command, ServiceCommand};
use crate::page_service::Services;
use crate::testing::{
    PageFixture, assert_header, assert_rows_hold_their_content, assert_rows_wait_while_busy,
    body_font, measured, rect_of,
};
use crate::units::{UnitListing, UnitLists, Units};

const AVAHI: &str = "avahi-daemon.service, Running";
const SSHD: &str = "sshd.service, Running";

type PageState = crate::testing::PageState<ServicesPage>;

fn listed(units: UnitListing) -> PageFixture {
    PageFixture {
        services: Services {
            units: Units::showing(UnitLists {
                system: units,
                user: UnitListing::NotLoaded,
            }),
            ..Services::default()
        },
        ..PageFixture::default()
    }
}

fn loaded(units: Vec<UnitInfo>) -> PageFixture {
    listed(UnitListing::Loaded(Arc::new(units)))
}

fn unit(name: &str, description: &str, file_state: FileState) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: description.to_owned(),
        active: ActiveState::Inactive,
        sub_state: "dead".to_owned(),
        file_state,
    }
}

fn sshd() -> UnitInfo {
    UnitInfo {
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        ..unit("sshd.service", "OpenSSH Daemon", FileState::Enabled)
    }
}

#[test]
fn before_the_first_list_placeholder_rows_wait() {
    let mut harness = listed(UnitListing::NotLoaded).harness(ServicesPage::default());

    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Reading the services…")
            .is_some()
    );
}

#[test]
fn a_new_list_keeps_the_last_units_on_screen() {
    let mut harness = listed(UnitListing::Loading {
        previous: Some(Arc::new(vec![sshd()])),
    })
    .harness(ServicesPage::default());

    // Read again spins while the list is read, so the frames never settle.
    harness.run_steps(3);

    assert!(harness.query_by_label("Reading the services…").is_none());
    assert!(harness.query_by_label_contains("OpenSSH Daemon").is_some());
}

#[test]
fn a_disabled_action_says_why_on_hover() {
    let masked = unit("cups.service", "CUPS", FileState::Masked);
    let mut harness = loaded(vec![masked]).harness(ServicesPage::default());

    harness.state_mut().page.filter.shown = Shown::All;

    harness.run();

    harness.get_by_label("Actions for cups.service").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::MenuItem, "Start")
        .hover();
    harness.run_steps(30);

    assert!(
        harness
            .query_by_label("A masked unit cannot start. Unmask it first.")
            .is_some()
    );
}

#[test]
fn the_scope_segments_pick_the_user_units() {
    let mut harness = listed(UnitListing::NotLoaded).harness(ServicesPage::default());

    harness.run();

    harness
        .get_by_role_and_label(Role::RadioButton, "User")
        .click();
    harness.run();

    assert_eq!(harness.state().page.scope, UnitScope::User);
}

fn sshd_listed() -> PageFixture {
    loaded(vec![sshd()])
}

#[test]
fn while_a_plan_runs_the_units_wait_and_a_banner_says_why() {
    assert_rows_wait_while_busy(sshd_listed, ServicesPage::default);
}

#[test]
fn while_a_plan_runs_the_whole_menu_waits() {
    for busy in [true, false] {
        let mut harness = PageFixture {
            busy,
            ..sshd_listed()
        }
        .harness(ServicesPage::default());

        harness.run();

        let menu = harness.get_by_role_and_label(Role::Button, "Actions for sshd.service");
        assert_eq!(menu.accesskit_node().is_disabled(), busy);
    }
}

#[test]
fn a_focused_switch_stays_with_its_unit_when_the_row_above_goes() {
    let avahi = UnitInfo {
        active: ActiveState::Active,
        ..unit("avahi-daemon.service", "Avahi", FileState::Enabled)
    };

    let mut harness = loaded(vec![avahi, sshd()]).harness(ServicesPage::default());

    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Start sshd.service at boot")
        .focus();
    harness.run();

    "sshd".clone_into(&mut harness.state_mut().page.filter.search);
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Start sshd.service at boot")
            .is_focused()
    );
}

fn two_listed() -> PageFixture {
    let avahi = UnitInfo {
        name: UnitName::new("avahi-daemon.service").unwrap(),
        description: "Avahi".to_owned(),
        ..sshd()
    };

    loaded(vec![avahi, sshd()])
}

fn focused(fixture: PageFixture, row: &str) -> egui_kittest::Harness<'static, PageState> {
    let mut harness = fixture.harness(ServicesPage::default());
    harness.run();
    assert!(tab_into(&mut harness, Role::ListItem));
    assert!(press_until(
        &mut harness,
        Key::ArrowDown,
        Role::ListItem,
        row
    ));
    harness
}

#[test]
fn down_moves_to_the_next_unit_and_stops_at_the_last() {
    let mut harness = focused(two_listed(), AVAHI);

    harness.key_press(Key::ArrowDown);
    harness.run();
    harness.key_press(Key::ArrowDown);
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::ListItem, SSHD)
            .is_focused()
    );
}

#[test]
fn space_on_a_focused_unit_flips_its_boot_switch() {
    let mut harness = focused(two_listed(), SSHD);

    harness.key_press(Key::Space);
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(ServiceCommand::Enable {
            scope: UnitScope::System,
            unit: UnitName::new("sshd.service").unwrap(),
            on: false,
        })]
    );
}

#[test]
fn the_menu_key_opens_the_unit_menu_and_escape_closes_it() {
    let mut harness = focused(two_listed(), SSHD);

    harness.key_press(Key::Menu);
    harness.run();
    harness
        .get_by_role_and_label(Role::MenuItem, "Show logs")
        .click();
    harness.run();
    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(ServiceCommand::Logs {
            scope: UnitScope::System,
            unit: UnitName::new("sshd.service").unwrap(),
        })]
    );

    harness.get_by_role_and_label(Role::ListItem, SSHD).focus();
    harness.run();
    harness.key_press_modifiers(Modifiers::SHIFT, Key::F10);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Stop")
            .is_some()
    );
    harness.key_press(Key::Escape);
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Stop")
            .is_none()
    );
}

#[test]
fn while_a_plan_runs_the_menu_key_opens_nothing() {
    // A row is focused before the plan starts, and a busy page drops that
    // focus, so the Menu key finds no row to open.
    let mut harness = focused(two_listed(), SSHD);
    harness.state_mut().fixture.busy = true;
    harness.run();
    assert!(
        !harness
            .get_by_role_and_label(Role::ListItem, SSHD)
            .is_focused()
    );

    harness.key_press(Key::Menu);
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Stop")
            .is_none()
    );
}

#[test]
fn a_frame_with_the_same_list_builds_no_rows() {
    let mut harness = two_listed().harness(ServicesPage::default());
    harness.run();
    let built = harness.state().page.rows.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.rows.builds(), built);

    harness.state_mut().page.filter.shown = Shown::All;
    harness.run();
    assert_eq!(harness.state().page.rows.builds(), built + 1);
}

#[test]
fn a_larger_desktop_font_makes_taller_rows_that_cut_no_text() {
    let mut harness = two_listed().harness(ServicesPage::default());
    body_font(&harness.ctx, 20.0);
    harness.run();

    assert_rows_hold_their_content(&harness, measured(20.0, row::height));
}

#[test]
fn refresh_sits_beside_the_title_and_the_toolbar_holds_search_and_sort() {
    let mut harness = sshd_listed().harness(ServicesPage::default());

    harness.run();

    let refresh = harness
        .get_by_role_and_label(Role::Button, "Read again")
        .rect();

    let search = harness
        .get_by_role_and_label(Role::TextInput, "Search services")
        .rect();

    let texts = (
        "Services",
        "Start, stop and enable the system's services and your own.",
    );

    assert_header(&harness, texts, Some(refresh), Some(search));
    let system = rect_of(&harness, Role::RadioButton, "System");
    assert!(system.right() < refresh.left(), "refresh is not last");
    let sort = rect_of(&harness, Role::ComboBox, "Sort services");
    assert!(
        (sort.right() - refresh.right()).abs() < 0.5,
        "sort {sort:?}, refresh {refresh:?}"
    );
    assert!(
        (sort.center().y - search.center().y).abs() < 0.5,
        "not one toolbar row"
    );
}
