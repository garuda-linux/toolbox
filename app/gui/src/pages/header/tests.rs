mod painted;

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_gui_kit::testing::painted_text;
use toolbox_gui_kit::widgets::{SEARCH_LEAST, tab_pill_inset, toolbar_end_inset};
use toolbox_plan::{UnitName, UnitScope};

use super::{
    Tabbed, TabbedIds, field_width, refresh_header_reading, search, tabbed_header_reading,
};
use crate::page_service::Services;
use crate::pages::services::ServicesPage;
use crate::testing::{PageFixture, PageState, rect_of};
use crate::units::{UnitListing, UnitLists, Units};

const IDS: TabbedIds<'static> = TabbedIds {
    title: "page-services",
    subtitle: "page-services-subtitle",
    tabs: "services-scope",
    refresh: "services-refresh",
};

/// Breeze joins two tabs, the one running a pixel under the next
/// (breezemetrics.h:135), so the gap between them is -1.
const TAB_JOIN: f32 = -1.0;
/// The caption "Sort by" and its dropdown are one control group.
const CAPTION_GAP: f32 = 4.0;
/// The gap between two groups of the toolbar: search, sort, tabs and refresh.
const GROUP_GAP: f32 = 8.0;
const WIDE: f32 = 1024.0;
const MEDIUM: f32 = 800.0;
/// The smallest window the app opens at.
const SMALLEST: f32 = 640.0;

fn tabbed() -> Harness<'static, Option<Tabbed<UnitScope>>> {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, done: &mut Option<Tabbed<UnitScope>>| {
            let scopes = [UnitScope::System, UnitScope::User];
            *done = Some(tabbed_header_reading(
                ui,
                IDS,
                &scopes,
                UnitScope::System,
                false,
            ));
        },
        None,
    );

    harness.run();
    harness
}

#[test]
fn a_tab_is_named_by_its_text_id() {
    let harness = tabbed();

    assert!(harness.query_by_label("System").is_some());
    assert!(harness.query_by_label("User").is_some());
}

#[test]
fn a_click_on_another_tab_is_handed_back() {
    let mut harness = tabbed();

    harness.get_by_label("User").click();
    harness.step();

    assert_eq!(
        *harness.state(),
        Some(Tabbed {
            picked: Some(UnitScope::User),
            refresh: false,
        })
    );
}

#[test]
fn f5_reads_again() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, pressed: &mut bool| {
            *pressed |= refresh_header_reading(
                ui,
                ("page-kernels", "kernels-subtitle"),
                "kernels-refresh",
                false,
            );
        },
        false,
    );

    harness.run();
    assert!(!*harness.state());

    harness.key_press(egui::Key::F5);
    harness.run();
    assert!(*harness.state());
}

#[test]
fn the_search_is_named_and_clears_with_the_shared_button() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, text: &mut String| _ = search(ui, text, "services-search", "services-search-hint"),
        "ssh".to_owned(),
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Clear Search")
        .click();
    harness.run();

    assert_eq!(harness.state(), "");
}

/// While a read runs Read again stays enabled, is named Reading, and a
/// click or F5 starts no second read.
#[test]
fn read_again_waits_while_a_read_runs() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, pressed: &mut bool| {
            *pressed |= refresh_header_reading(
                ui,
                ("page-kernels", "kernels-subtitle"),
                "kernels-refresh",
                true,
            );
        },
        false,
    );

    harness.run_steps(3);
    let button = harness.get_by_role_and_label(Role::Button, "Reading");
    assert!(!button.accesskit_node().is_disabled());

    button.click();
    harness.run_steps(3);
    assert!(!*harness.state());

    harness.key_press(egui::Key::F5);
    harness.run_steps(3);
    assert!(!*harness.state());
}

#[test]
fn a_search_keeps_its_wanted_width_down_to_its_least() {
    assert!((field_width(900.0, 240.0) - 240.0).abs() < f32::EPSILON);
    assert!((field_width(220.0, 240.0) - 220.0).abs() < f32::EPSILON);
    assert!((field_width(40.0, 240.0) - SEARCH_LEAST).abs() < f32::EPSILON);
}

#[test]
fn a_tabbed_read_again_waits_while_a_read_runs() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, done: &mut Option<Tabbed<UnitScope>>| {
            let scopes = [UnitScope::System, UnitScope::User];
            *done = Some(tabbed_header_reading(
                ui,
                IDS,
                &scopes,
                UnitScope::System,
                true,
            ));
        },
        None,
    );

    harness.run_steps(3);
    harness.key_press(egui::Key::F5);
    harness.run_steps(3);

    assert!(harness.state().is_some_and(|done| !done.refresh));
}

fn services_header(width: f32) -> Harness<'static, PageState<ServicesPage>> {
    let mut harness = services_listed().harness(ServicesPage::default());
    harness.set_size(vec2(width, 900.0));
    harness.run();
    harness
}

fn services_listed() -> PageFixture {
    let sshd = UnitInfo {
        name: UnitName::new("sshd.service").unwrap(),
        description: toolbox_text::plain("OpenSSH Daemon"),
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        file_state: FileState::Enabled,
    };

    PageFixture {
        services: Services {
            units: Units::showing(UnitLists {
                system: UnitListing::Loaded(Arc::new(vec![sshd])),
                user: UnitListing::NotLoaded,
            }),
            ..Services::default()
        },
        ..PageFixture::default()
    }
}

fn same_row(one: Rect, other: Rect) -> bool {
    (one.center().y - other.center().y).abs() < 1.0
}

#[track_caller]
fn assert_gap(width: f32, between: &str, gap: f32, wanted: f32) {
    assert!(
        (gap - wanted).abs() < 0.5,
        "{between} at {width} points is {gap} px, wanted {wanted}"
    );
}

/// Read again touches the tabs, as 8 px between what both paint would
/// overlap their rects.
#[test]
fn the_tabs_join_and_read_again_touches_them() {
    for width in [WIDE, SMALLEST] {
        let harness = services_header(width);
        let system = rect_of(&harness, Role::RadioButton, "System");
        let user = rect_of(&harness, Role::RadioButton, "User");
        let refresh = rect_of(&harness, Role::Button, "Read Again");

        assert_gap(
            width,
            "System to User",
            user.left() - system.right(),
            TAB_JOIN,
        );
        assert_gap(
            width,
            "User to Read again",
            refresh.left() - user.right(),
            0.0,
        );
    }
}

/// The sort stands short of the tabs by what both paint inside their rects,
/// so the eye sees the group gap.
#[test]
fn the_sort_rect_stands_the_painted_insets_short_of_the_tabs() {
    for width in [WIDE, MEDIUM, SMALLEST] {
        let harness = services_header(width);
        let search = rect_of(&harness, Role::TextInput, "Search services");
        let sort = rect_of(&harness, Role::ComboBox, "Sort services");
        let system = rect_of(&harness, Role::RadioButton, "System");

        if same_row(sort, system) {
            assert_gap(
                width,
                "sort to the tabs",
                system.left() - sort.right(),
                GROUP_GAP - toolbar_end_inset() - tab_pill_inset(),
            );
        }

        // The gap to the sort is measured on the painted shapes in `painted`.
        if same_row(search, sort) {
            assert!(
                search.width() >= SEARCH_LEAST - 0.5,
                "search at {width} points is {} px, wanted at least {SEARCH_LEAST}",
                search.width()
            );
        }
    }
}

fn sort_caption<S>(harness: &Harness<'_, S>) -> Rect {
    painted_text(harness, "Sort by")
        .expect("the sort caption is painted")
        .visual_bounding_rect()
}

#[test]
fn the_sort_caption_sits_four_points_from_its_dropdown() {
    for width in [WIDE, SMALLEST] {
        let harness = services_header(width);
        let caption = sort_caption(&harness);
        let dropdown = rect_of(&harness, Role::ComboBox, "Sort services");

        assert_gap(
            width,
            "Sort by to its dropdown",
            dropdown.left() - caption.right(),
            CAPTION_GAP,
        );
    }
}

#[test]
fn the_sort_shares_the_tabs_row_on_the_wide_window() {
    let harness = services_header(WIDE);
    let system = rect_of(&harness, Role::RadioButton, "System");
    let sort = rect_of(&harness, Role::ComboBox, "Sort services");

    assert!(
        same_row(sort, system),
        "sort {sort:?} is not beside the tabs {system:?}"
    );
}
