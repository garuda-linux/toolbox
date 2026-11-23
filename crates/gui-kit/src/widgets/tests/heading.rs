use egui::Ui;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{
    SectionHeading, icon_button, icon_button_width, page_header, page_toolbar, pill_button,
};
use crate::testing::{harness, spoken_words};
use crate::widgets::ButtonKind;
use crate::{UiExt as _, icons};

const TABS: [&str; 4] = ["Status", "Hardware", "Drivers", "Tools"];

#[test]
fn trailing_actions_sit_at_the_heading_and_take_clicks() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, clicked: &mut bool| {
            SectionHeading::new("Installed", "3 kernels")
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

/// A header with a search-sized button and a sort-sized one in its
/// toolbar, `width` wide.
fn toolbar(width: f32) -> egui_kittest::Harness<'static> {
    let mut harness = harness(width, 300.0).build_ui(|ui| {
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

/// KDE apps (Discover, Dolphin) put a page's search and sort into its
/// toolbar, at the right of the title.
#[test]
fn the_toolbar_sits_in_the_title_row_at_its_right_end() {
    let harness = toolbar(600.0);
    let column = harness.ctx.content_rect();
    let search = harness.get_by_role_and_label(Role::Button, "Search").rect();
    let sort = harness.get_by_role_and_label(Role::Button, "Sort").rect();
    let title = harness.get_by_label("Services").rect();

    assert!(sort.right() <= column.right() + 0.5);
    assert!(
        sort.right() > column.right() - 20.0,
        "sort is not at the right edge"
    );
    assert!(search.right() <= sort.left(), "search not before the sort");
    assert!(search.left() > title.right(), "search over the title");
    for control in [search, sort] {
        assert!(
            (control.center().y - title.center().y).abs() < 1.0,
            "not on the title's row"
        );
    }
}

#[test]
fn a_toolbar_without_room_beside_the_title_takes_its_own_row() {
    let harness = toolbar(200.0);
    let search = harness.get_by_role_and_label(Role::Button, "Search").rect();
    let title = harness.get_by_label("Services").rect();

    assert!(search.top() > title.bottom());
}

#[test]
fn an_icon_button_is_as_wide_as_its_width_says() {
    let widths = std::rc::Rc::new(std::cell::Cell::new((0.0_f32, 0.0_f32)));
    let seen = std::rc::Rc::clone(&widths);
    let mut harness = harness(300.0, 100.0).build_ui(move |ui| {
        let drawn = icon_button(ui, icons::VIEW_REFRESH, "Refresh").rect.width();
        seen.set((icon_button_width(), drawn));
    });

    harness.run();
    let (said, drawn) = widths.get();
    assert!((said - drawn).abs() < 0.5, "said {said}, drawn {drawn}");
}

fn tab_row(ui: &mut Ui) {
    let options: Vec<(usize, &str)> = TABS.iter().copied().enumerate().collect();
    _ = super::super::segmented(ui, "Tabs", "Tabs", &options, 0);
    ui.add_space(8.0);
}

fn tabs_width(ui: &Ui) -> f32 {
    super::super::segmented_width(ui, &TABS) + 8.0 + icon_button_width()
}

/// Where a header's title and its refresh button land, with or without
/// tabs before the refresh and with a one or two line purpose.
fn refresh_of(subtitle: &'static str, tabs: bool) -> (egui::Rect, egui::Rect) {
    let mut harness = harness(700.0, 300.0).build_ui(move |ui| {
        let width = if tabs {
            tabs_width(ui)
        } else {
            icon_button_width()
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
        SectionHeading::new("Radios", "").show(ui);
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
            if super::super::refresh_button(ui, "Read again")
                .value
                .is_some()
            {
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
            ui.enabled_ui(false, |ui| {
                if super::super::refresh_button(ui, "Read again")
                    .value
                    .is_some()
                {
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

/// A header `width` wide with four tabs and Refresh, and a purpose that
/// is not drawn in the toolbar row. Returns the title and the tabs.
fn tabbed_header(width: f32, title: &'static str) -> (egui::Rect, egui::Rect) {
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

    (harness.get_by_label(title).rect(), tabs)
}

/// The purpose heads the body ([`page_layout_shaped`]), so the toolbar row
/// holds the title and the tabs, which fit its height.
#[test]
fn the_tabs_fit_the_toolbar_row() {
    let (_, tabs) = tabbed_header(800.0, "Maintenance");
    let row = crate::tokens::Sizes::DEFAULT.toolbar_height;

    assert!(tabs.height() <= row, "the tabs {tabs:?} outgrow the row");
}

#[test]
fn the_tabs_stay_on_the_title_line_while_the_title_fits_beside_them() {
    let (title, tabs) = tabbed_header(560.0, "Boot");

    assert!(
        (tabs.center().y - title.center().y).abs() < 0.5,
        "the tabs {tabs:?} left the title's line {title:?}"
    );
}

#[test]
fn a_section_intro_leaves_no_empty_label() {
    let mut harness = harness(600.0, 200.0).build_ui(|ui| {
        super::super::section_intro(ui, "Installed", "The kernels on disk.");
    });

    harness.run();

    let empty = harness
        .query_all_by_role(Role::Label)
        .filter(|node| spoken_words(node).is_empty())
        .count();

    assert_eq!(empty, 0, "a label with no words");
}
