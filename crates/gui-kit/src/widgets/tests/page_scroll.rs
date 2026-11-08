use egui::Ui;
use egui_kittest::kittest::Queryable as _;

use super::page_layout;
use crate::testing::harness;

/// A long page whose layout salt is the state, so a test can switch it
/// as a new page or tab would.
fn long_page() -> egui_kittest::Harness<'static, u8> {
    let mut harness = harness(400.0, 300.0).build_ui_state(
        |root, place: &mut u8| {
            let salt = *place;
            page_layout(
                root,
                salt,
                &mut (),
                |(), ui: &mut Ui| {
                    ui.label("Title");
                    true
                },
                |(), ui: &mut Ui| {
                    for row in 0_u8..60 {
                        ui.label(format!("Row {row}"));
                    }
                },
            );
        },
        0,
    );

    harness.run();
    harness
}

fn scroll_down(harness: &mut egui_kittest::Harness<'static, u8>) {
    harness.event(egui::Event::PointerMoved(egui::pos2(200.0, 200.0)));
    harness.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -600.0),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    harness.run();
}

fn first_row_top(harness: &egui_kittest::Harness<'static, u8>) -> f32 {
    harness.get_by_label("Row 0").rect().top()
}

#[test]
fn another_page_or_tab_starts_at_the_top() {
    let mut harness = long_page();
    let top = first_row_top(&harness);
    scroll_down(&mut harness);
    assert!(
        first_row_top(&harness) < top - 100.0,
        "the wheel did not scroll"
    );

    *harness.state_mut() = 1;
    harness.run();
    assert!(
        (first_row_top(&harness) - top).abs() < 0.5,
        "the new place kept the scroll"
    );

    scroll_down(&mut harness);
    *harness.state_mut() = 0;
    harness.run();
    assert!(
        (first_row_top(&harness) - top).abs() < 0.5,
        "going back kept the scroll"
    );
}

#[test]
fn the_same_place_keeps_its_scroll() {
    let mut harness = long_page();
    scroll_down(&mut harness);
    let scrolled = first_row_top(&harness);

    harness.run();
    harness.run();
    assert!((first_row_top(&harness) - scrolled).abs() < 0.5);
}

#[test]
fn a_covered_bottom_leaves_room_to_scroll_every_row_above_the_cover() {
    const COVER: f32 = 150.0;
    let mut harness = harness(400.0, 300.0).build_ui(|root| {
        super::super::cover_page_bottom(root.ctx(), COVER);
        page_layout(
            root,
            "covered",
            &mut (),
            |(), ui: &mut Ui| {
                ui.label("Title");
                true
            },
            |(), ui: &mut Ui| {
                for row in 0_u8..30 {
                    ui.label(format!("Row {row}"));
                }
            },
        );
    });

    harness.run();

    for _ in 0_u8..4 {
        harness.event(egui::Event::PointerMoved(egui::pos2(200.0, 100.0)));
        harness.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -2000.0),
            modifiers: egui::Modifiers::NONE,
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
        });
        harness.run_steps(12);
    }

    let last = harness.get_by_label("Row 29").rect();
    assert!(
        last.bottom() <= 300.0 - COVER,
        "the last row ends at {} under a cover from {}",
        last.bottom(),
        300.0 - COVER
    );
}
