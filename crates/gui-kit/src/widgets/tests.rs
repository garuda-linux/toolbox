mod action_row;
mod action_row_tile;
mod banner;
mod centred;
mod check_row;
mod choices;
mod contrast;
mod cut;
mod dialog_edges;
mod dialog_focus;
mod edge_fade;
mod feedback;
mod field_rim;
mod focus_order;
mod footer;
mod frosted;
mod frosted_header;
mod heading;
mod highlight;
mod icon_tile;
mod layout;
mod letter_tile;
mod link_row;
mod list;
mod live_header;
mod metric;
mod modal;
mod motion;
mod page_cover;
mod page_entry;
mod page_scroll;
mod press;
mod property;
mod question;
mod raised;
mod removable;
mod ring;
mod roles;
mod row_centre;
mod row_controls;
mod row_fill;
mod search;
mod select_width;
mod selection;
mod slide;
mod switch_rim;
mod text_box;
mod text_link;
mod title_gap;
mod wrapped_title;

use egui::accesskit::Role;
use egui::{Theme, Ui};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, SnapshotResults};

use super::{
    ButtonKind, NavLook, SearchTexts, Select, Tone, card, card_row, chip, group_label, link_tile,
    menu_button, menu_heading, menu_item, nav_row, page_header, pill_button, search_field,
    section_heading, segmented, switch, tag,
};
use crate::icons;
use crate::testing::harness;

const EXPANDED: NavLook = NavLook {
    expanded: true,
    height: 32.0,
};

#[derive(Default)]
struct Gallery {
    on: bool,
    clicks: usize,
    followed: usize,
    search: String,
}

fn gallery(ui: &mut Ui, state: &mut Gallery) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(220.0);
            group_label(ui, "System", false);
            nav_row(ui, icons::GO_HOME, "Home", false, EXPANDED);
            nav_row(ui, icons::PLUGINS, "Components", true, EXPANDED);
            nav_row(ui, icons::INPUT_GAMEPAD, "Gaming", false, EXPANDED);
        });

        ui.vertical(|ui| page(ui, state));
    });
}

fn page(ui: &mut Ui, state: &mut Gallery) {
    page_header(
        ui,
        "Components",
        "Hardware support and system features.",
        248.0,
        |ui| {
            let texts = SearchTexts {
                name: "Search components",
                hint: "Search components",
                clear: "Clear the search",
            };

            search_field(ui, &mut state.search, texts, 220.0);
        },
    );
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        chip(ui, "All", Some("3/12"), true);
        chip(ui, "Audio", Some("2/8"), false);
    });
    ui.add_space(12.0);
    section_heading(ui, icons::APPLICATIONS_MULTIMEDIA, "Audio", "2/8");
    card(ui, |ui| {
        card_row(ui, true, |ui| {
            ui.horizontal(|ui| {
                ui.label("PipeWire");
                tag(ui, "Turns on with Apply", Tone::Accent);
                if switch(ui, state.on, "PipeWire", true).clicked() {
                    state.on = !state.on;
                }
            });
        });
        card_row(ui, false, |ui| {
            ui.horizontal(|ui| {
                ui.label("JamesDSP");
                tag(ui, "Needs PipeWire", Tone::Neutral);
                switch(ui, false, "JamesDSP", false);
            });
        });
    });
    ui.add_space(12.0);
    actions(ui, state);
}

fn actions(ui: &mut Ui, state: &mut Gallery) {
    ui.horizontal(|ui| {
        pill_button(ui, "Review", ButtonKind::Ghost);
        pill_button(ui, "Remove all", ButtonKind::Soft);
        if pill_button(ui, "Apply", ButtonKind::Primary).clicked() {
            state.clicks += 1;
        }
    });
    ui.add_space(12.0);
    if link_tile(
        ui,
        icons::TOOLS,
        "Maintenance",
        "Keep your system in a good state, with a subtitle long enough to be cut",
        300.0,
    )
    .clicked()
    {
        state.followed += 1;
    }
}

fn app(theme: Theme) -> Harness<'static, Gallery> {
    let mut harness = harness(820.0, 440.0)
        .with_theme(theme)
        .build_ui_state(gallery, Gallery::default());

    harness.run();
    harness
}

#[test]
fn a_switch_is_a_named_check_box_that_reports_clicks() {
    let mut harness = app(Theme::Dark);

    harness
        .get_by_role_and_label(Role::CheckBox, "PipeWire")
        .click();
    harness.run();

    assert!(harness.state().on);
}

#[test]
fn a_disabled_switch_ignores_clicks() {
    let mut harness = app(Theme::Dark);
    let james = harness.get_by_role_and_label(Role::CheckBox, "JamesDSP");
    assert!(james.accesskit_node().is_disabled());

    james.click();
    harness.run();

    assert!(!harness.state().on);
}

#[test]
fn buttons_and_chips_are_named_buttons_and_nav_rows_tabs() {
    let mut harness = app(Theme::Dark);

    harness.get_by_role_and_label(Role::Button, "Apply").click();
    harness.run();

    assert_eq!(harness.state().clicks, 1);
    for (role, label) in [
        (Role::Button, "All 3/12"),
        (Role::Button, "Audio 2/8"),
        (Role::Tab, "Gaming"),
    ] {
        let found = harness.query_by_role_and_label(role, label);
        assert!(found.is_some(), "{label}");
    }

    assert!(harness.query_by_label("Needs PipeWire").is_some());
}

#[test]
fn the_search_field_is_named_and_takes_text() {
    let mut harness = app(Theme::Dark);

    let field = harness.get_by_role_and_label(Role::TextInput, "Search components");
    field.focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Search components")
        .type_text("pipe");
    harness.run();

    assert_eq!(harness.state().search, "pipe");
}

#[test]
fn the_kit_in_both_themes() {
    let mut results = SnapshotResults::new();

    for (theme, name) in [
        (Theme::Dark, "widgets/gallery_dark"),
        (Theme::Light, "widgets/gallery_light"),
    ] {
        let mut harness = app(theme);
        harness.get_by_label("Home").hover();
        harness.run();
        results.add(harness.try_snapshot(name));
    }
}

#[test]
fn a_link_tile_is_a_named_link() {
    let mut harness = app(Theme::Dark);

    harness
        .get_by_role_and_label(Role::Link, "Maintenance")
        .click();
    harness.run();

    assert_eq!(harness.state().followed, 1);
}

#[test]
fn a_select_opens_its_options_and_reports_the_pick() {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |ui, picked: &mut Option<usize>| {
            let options = ["Bash", "Zsh", "Fish"];
            if let Some(index) = Select::new("Login shell", &options)
                .selected(Some(0))
                .placeholder("None")
                .show(ui)
            {
                *picked = Some(index);
            }
        },
        None,
    );

    harness.run();
    assert!(harness.query_by_label("Zsh").is_none());

    harness
        .get_by_role_and_label(Role::ComboBox, "Login shell")
        .click();
    harness.run();
    harness.get_by_label("Zsh").click();
    harness.run();

    assert_eq!(*harness.state(), Some(1));
    assert!(harness.query_by_label("Zsh").is_none(), "the list closed");
}

#[test]
fn a_pick_without_the_pointer_closes_the_list_too() {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |ui, picked: &mut Option<usize>| {
            let options = ["Bash", "Zsh", "Fish"];
            if let Some(index) = Select::new("Login shell", &options)
                .selected(Some(0))
                .show(ui)
            {
                *picked = Some(index);
            }
        },
        None,
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::ComboBox, "Login shell")
        .click_accesskit();
    harness.run();
    harness.get_by_label("Fish").click_accesskit();
    harness.run();

    assert_eq!(*harness.state(), Some(2));
    assert!(harness.query_by_label("Fish").is_none(), "the list closed");
}

#[test]
fn segments_are_radio_buttons_and_report_the_pick() {
    let mut harness = harness(400.0, 120.0).build_ui_state(
        |ui, picked: &mut Option<usize>| {
            if let Some(index) = segmented(ui, "Units", &[(0, "System"), (1, "User")], 0) {
                *picked = Some(index);
            }
        },
        None,
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::RadioButton, "User")
        .click();
    harness.run();

    assert_eq!(*harness.state(), Some(1));
}

#[test]
fn a_menu_opens_its_items_and_reports_a_click() {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |ui, clicked: &mut bool| {
            menu_button(ui, "Actions for sshd", |ui| {
                menu_heading(ui, "Now");
                if menu_item(ui, "Restart").clicked() {
                    *clicked = true;
                }
            });
        },
        false,
    );

    harness.run();
    assert!(harness.query_by_label("Restart").is_none());

    harness.get_by_label("Actions for sshd").click();
    harness.run();
    harness
        .get_by_role_and_label(Role::MenuItem, "Restart")
        .click();
    harness.run();

    assert!(*harness.state());
    assert!(
        harness.query_by_label("Restart").is_none(),
        "the menu closed"
    );
}

fn page_layout<P>(
    ui: &mut Ui,
    salt: impl std::hash::Hash + std::fmt::Debug,
    page: &mut P,
    header: impl FnOnce(&mut P, &mut Ui) -> bool,
    body: impl FnOnce(&mut P, &mut Ui),
) {
    let parts = super::PageParts {
        header,
        body,
        footer: |_: &mut P, _: &mut Ui| false,
    };

    super::page_layout_shaped(ui, salt, page, parts, super::BodyShape::Column);
}

#[test]
fn a_menu_item_picked_without_the_pointer_closes_the_menu() {
    let mut harness = harness(300.0, 300.0).build_ui_state(
        |ui, picked: &mut bool| {
            menu_button(ui, "More actions", |ui| {
                *picked |= menu_item(ui, "Copy").clicked();
            });
        },
        false,
    );

    harness.run();
    harness.get_by_label("More actions").click_accesskit();
    harness.run();
    harness.get_by_label("Copy").click_accesskit();
    harness.run();

    assert!(*harness.state());
    assert!(
        harness.query_by_label("Copy").is_none(),
        "the menu stayed open"
    );
}
