use egui::Ui;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{
    SectionHeading, icon_button, icon_button_width, page_header, page_toolbar, pill_button,
};
use crate::icons;
use crate::testing::harness;
use crate::widgets::ButtonKind;

const TABS: [&str; 4] = ["Status", "Hardware", "Drivers", "Tools"];

#[test]
fn trailing_actions_sit_at_the_heading_and_take_clicks() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, clicked: &mut bool| {
            SectionHeading::new(icons::RUN_BUILD, "Installed", "3 kernels")
                .trailing(80.0, |actions| {
                    if pill_button(actions, "Refresh", ButtonKind::Soft).clicked() {
                        *clicked = true;
                    }
                })
                .show(ui);
        },
        false,
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Refresh")
        .click();
    harness.run();

    assert!(*harness.state());
}

/// A header with a search-sized button at the left of the toolbar and a
/// sort-sized one at the right, in a 600 wide column.
fn toolbar() -> egui_kittest::Harness<'static> {
    let mut harness = harness(600.0, 300.0).build_ui(|ui| {
        page_header(ui, "Services", "Units of the system.", 0.0, |_| {});
        page_toolbar(
            ui,
            &mut (),
            |(), ui| _ = pill_button(ui, "Search", ButtonKind::Soft),
            |(), ui| _ = pill_button(ui, "Sort", ButtonKind::Soft),
        );
    });

    harness.run();
    harness
}

#[test]
fn the_toolbar_puts_its_ends_at_the_edges_of_the_column() {
    let harness = toolbar();
    let column = harness.ctx.content_rect();
    let search = harness.get_by_role_and_label(Role::Button, "Search").rect();
    let sort = harness.get_by_role_and_label(Role::Button, "Sort").rect();
    let title = harness.get_by_label("Services").rect();

    assert!((search.left() - title.left()).abs() < 0.5);
    assert!(sort.right() <= column.right() + 0.5);
    assert!(
        sort.right() > column.right() - 20.0,
        "sort is not at the right edge"
    );
    assert!(
        (search.center().y - sort.center().y).abs() < 0.5,
        "not one row"
    );
    assert!(search.top() > title.bottom());
}

#[test]
fn an_icon_button_is_as_wide_as_its_width_says() {
    let widths = std::rc::Rc::new(std::cell::Cell::new((0.0_f32, 0.0_f32)));
    let seen = std::rc::Rc::clone(&widths);
    let mut harness = harness(300.0, 100.0).build_ui(move |ui| {
        let drawn = icon_button(ui, icons::VIEW_REFRESH, "Refresh").rect.width();
        seen.set((icon_button_width(ui), drawn));
    });

    harness.run();
    let (said, drawn) = widths.get();
    assert!((said - drawn).abs() < 0.5, "said {said}, drawn {drawn}");
}

fn tab_row(ui: &mut Ui) {
    let options: Vec<(usize, &str)> = TABS.iter().copied().enumerate().collect();
    _ = super::super::segmented(ui, "Tabs", &options, 0);
    ui.add_space(8.0);
}

fn tabs_width(ui: &Ui) -> f32 {
    super::super::segmented_width(ui, &TABS) + 8.0 + icon_button_width(ui)
}

/// Where a header's title and its refresh button land, with or without
/// tabs before the refresh and with a one or two line purpose.
fn refresh_of(subtitle: &'static str, tabs: bool) -> (egui::Rect, egui::Rect) {
    let mut harness = harness(700.0, 300.0).build_ui(move |ui| {
        let width = if tabs {
            tabs_width(ui)
        } else {
            icon_button_width(ui)
        };

        page_header(ui, "Network", subtitle, width, |ui| {
            if tabs {
                tab_row(ui);
            }

            _ = icon_button(ui, icons::VIEW_REFRESH, "Refresh");
        });
    });

    harness.run();
    let title = harness.get_by_label("Network").rect();
    let refresh = harness
        .get_by_role_and_label(Role::Button, "Refresh")
        .rect();

    (title, refresh)
}

#[test]
fn the_refresh_button_sits_in_one_spot_with_or_without_tabs_and_long_purposes() {
    let short = "Files of the system.";
    let long = "Connections, devices, drivers and tools for the network, \
                with a purpose long enough to wrap onto a second line here.";

    let (title, plain) = refresh_of(short, false);

    for (subtitle, tabs) in [(short, true), (long, false), (long, true)] {
        let (other_title, refresh) = refresh_of(subtitle, tabs);
        assert!((other_title.top() - title.top()).abs() < 0.5);
        assert!(
            (refresh.center().y - plain.center().y).abs() < 0.5
                && (refresh.right() - plain.right()).abs() < 0.5,
            "tabs {tabs}, purpose {subtitle:?}: refresh at {refresh:?}, alone at {plain:?}"
        );
    }

    assert!(
        (plain.center().y - title.center().y).abs() < 0.5,
        "the refresh is not level with the title: {plain:?} against {title:?}"
    );
}

#[test]
fn a_narrow_header_keeps_the_refresh_after_the_tabs() {
    let mut harness = harness(420.0, 300.0).build_ui(|ui| {
        let width = tabs_width(ui);
        page_header(
            ui,
            "Network",
            "The connection and the tools.",
            width,
            |ui| {
                tab_row(ui);
                _ = icon_button(ui, icons::VIEW_REFRESH, "Refresh");
            },
        );
    });

    harness.run();
    let refresh = harness
        .get_by_role_and_label(Role::Button, "Refresh")
        .rect();

    let tools = harness
        .get_by_role_and_label(Role::RadioButton, "Tools")
        .rect();

    let title = harness.get_by_label("Network").rect();

    assert!(
        refresh.left() > tools.right(),
        "refresh {refresh:?} before the tabs {tools:?}"
    );
    assert!(
        refresh.top() > title.bottom(),
        "the controls are not stacked"
    );
}

#[test]
fn page_titles_and_section_titles_are_headings_for_screen_readers() {
    let mut harness = harness(700.0, 300.0).build_ui(|ui| {
        page_header(ui, "Network", "The connection.", 40.0, |_| {});
        SectionHeading::new(icons::RUN_BUILD, "Radios", "").show(ui);
    });

    harness.run();

    let page = harness.get_by_label("Network");
    assert_eq!(page.accesskit_node().role(), Role::Heading);
    assert_eq!(page.accesskit_node().level(), Some(1));

    let section = harness.get_by_label("Radios");
    assert_eq!(section.accesskit_node().role(), Role::Heading);
    assert_eq!(section.accesskit_node().level(), Some(2));
}

#[test]
fn f5_reads_the_page_again_like_its_refresh_button() {
    let mut harness = harness(400.0, 200.0).build_ui_state(
        |ui, read: &mut u32| {
            if super::super::refresh_button(ui, "Read again") {
                *read += 1;
            }
        },
        0,
    );

    harness.run();
    harness.key_press(egui::Key::F5);
    harness.run();

    assert_eq!(*harness.state(), 1);
}

#[test]
fn f5_leaves_a_disabled_refresh_button_alone() {
    let mut harness = harness(400.0, 200.0).build_ui_state(
        |ui, read: &mut u32| {
            ui.add_enabled_ui(false, |ui| {
                if super::super::refresh_button(ui, "Read again") {
                    *read += 1;
                }
            });
        },
        0,
    );

    harness.run();
    harness.key_press(egui::Key::F5);
    harness.run();

    assert_eq!(*harness.state(), 0);
}

/// A header `width` wide with four tabs and Refresh, and a purpose long
/// enough to wrap beside them. Returns the title, the purpose and the tabs.
fn tabbed_header(width: f32, title: &'static str) -> (egui::Rect, egui::Rect, egui::Rect) {
    const PURPOSE: &str = "Keep the system in a good state: updates, cleanups, tools and repairs.";
    let mut harness = harness(width, 300.0).build_ui(move |ui| {
        let controls = tabs_width(ui);
        page_header(ui, title, PURPOSE, controls, |ui| {
            tab_row(ui);
            _ = icon_button(ui, icons::VIEW_REFRESH, "Refresh");
        });
    });

    harness.run();
    let tabs = harness
        .get_by_role_and_label(Role::RadioButton, "Status")
        .rect()
        .union(
            harness
                .get_by_role_and_label(Role::RadioButton, "Tools")
                .rect(),
        );

    (
        harness.get_by_label(title).rect(),
        harness.get_by_label(PURPOSE).rect(),
        tabs,
    )
}

#[test]
fn the_purpose_runs_under_the_tabs_to_the_column_edge() {
    let (_, purpose, tabs) = tabbed_header(800.0, "Maintenance");

    assert!(
        purpose.right() > tabs.left(),
        "the purpose {purpose:?} wraps short of the tabs {tabs:?}"
    );
    assert!(
        purpose.top() >= tabs.bottom(),
        "the purpose {purpose:?} runs into the tabs {tabs:?}"
    );
}

#[test]
fn the_tabs_stay_on_the_title_line_while_the_title_fits_beside_them() {
    let (title, _, tabs) = tabbed_header(560.0, "Boot");

    assert!(
        (tabs.center().y - title.center().y).abs() < 0.5,
        "the tabs {tabs:?} left the title's line {title:?}"
    );
}
