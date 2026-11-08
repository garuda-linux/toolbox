use egui::Ui;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::page_layout;
use crate::testing::{harness, painted_text};

fn page() -> egui_kittest::Harness<'static, u8> {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |root, page: &mut u8| {
            page_layout(
                root,
                "page",
                page,
                |_, ui: &mut Ui| {
                    _ = ui.button("Search");
                    true
                },
                |_, ui: &mut Ui| {
                    _ = ui.button("First row");
                    _ = ui.button("Second row");
                },
            );
        },
        0,
    );

    harness.run();
    harness
}

fn focused(harness: &egui_kittest::Harness<'static, u8>) -> Option<String> {
    let node = harness
        .query_all(egui_kittest::kittest::by())
        .find(|node| node.accesskit_node().is_focused())?;

    node.accesskit_node().label()
}

fn tab(harness: &mut egui_kittest::Harness<'static, u8>) {
    harness.key_press(egui::Key::Tab);
    harness.run();
}

#[test]
fn tab_reaches_the_header_before_the_body() {
    let mut harness = page();

    tab(&mut harness);
    assert_eq!(focused(&harness).as_deref(), Some("Search"));

    tab(&mut harness);
    assert_eq!(focused(&harness).as_deref(), Some("First row"));
}

#[test]
fn the_header_band_is_no_tab_stop_of_its_own() {
    let mut harness = page();
    let mut stops = Vec::new();

    for _ in 0_u8..4 {
        tab(&mut harness);
        stops.push(focused(&harness));
    }

    assert!(
        stops.iter().all(Option::is_some),
        "a Tab stop without a name: {stops:?}"
    );
}

fn row() -> egui_kittest::Harness<'static, u8> {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, _: &mut u8| {
            super::super::split_row(
                ui,
                200.0,
                |ui| _ = ui.label("A row"),
                |ui| {
                    _ = ui.button("Left");
                    _ = ui.button("Right");
                },
            );
        },
        0,
    );

    harness.run();
    harness
}

#[test]
fn a_rows_controls_take_tab_in_the_order_they_are_seen() {
    let mut harness = row();

    let left = harness.get_by_label("Left").rect();
    let right = harness.get_by_label("Right").rect();
    assert!(left.max.x <= right.min.x, "Left sits left of Right");

    tab(&mut harness);
    assert_eq!(focused(&harness).as_deref(), Some("Left"));
    tab(&mut harness);
    assert_eq!(focused(&harness).as_deref(), Some("Right"));
}

#[test]
fn a_rows_controls_end_at_its_right_edge() {
    let harness = row();

    let right = harness.get_by_label("Right").rect();
    assert!(right.max.x > 580.0, "ends at {}", right.max.x);
}

#[test]
fn a_rows_value_that_fits_is_never_cut() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, _: &mut u8| {
            super::super::split_row(
                ui,
                200.0,
                |ui| _ = ui.label("Router"),
                |ui| _ = ui.add(egui::Label::new("192.168.1.1").truncate()),
            );
        },
        0,
    );

    harness.run();
    let shown = painted_text(&harness, "192.168.1.1").expect("the value is painted");
    assert!(!shown.galley.elided);
}

#[test]
fn a_rows_value_that_grows_is_never_left_cut() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, long: &mut bool| {
            let value = if *long {
                "Not known at all"
            } else {
                "Reading…"
            };

            super::super::split_row(
                ui,
                200.0,
                |ui| _ = ui.label("Router"),
                |ui| _ = ui.add(egui::Label::new(value).truncate()),
            );
        },
        false,
    );

    harness.run();
    *harness.state_mut() = true;
    harness.run();

    let shown = painted_text(&harness, "Not known at all").expect("the value is painted");
    assert!(!shown.galley.elided);
}
