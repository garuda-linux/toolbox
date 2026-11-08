mod changed_controls;

use egui::{Rect, vec2};

use super::super::{MIN_TEXT_WIDTH, split_row, status_dot, text_block_height, title_line};
use crate::testing::harness;
use crate::widgets::Tone;

#[derive(Clone, Copy)]
struct Placed {
    text: Rect,
    controls: Rect,
    right: f32,
}

fn split(width: f32) -> Placed {
    let mut harness = harness(width, 300.0).build_ui_state(
        |ui, placed: &mut Placed| {
            placed.right = ui.max_rect().right();
            let (text, controls) = split_row(
                ui,
                200.0,
                |ui| {
                    ui.label("PipeWire, the sound server of modern desktops")
                        .rect
                },
                |ui| {
                    ui.allocate_exact_size(vec2(200.0, 30.0), egui::Sense::click())
                        .0
                },
            );

            placed.text = text;

            placed.controls = controls;
        },
        Placed {
            text: Rect::NOTHING,
            controls: Rect::NOTHING,
            right: 0.0,
        },
    );

    harness.run();
    *harness.state()
}

#[test]
fn on_a_wide_row_the_controls_sit_beside_the_text() {
    let Placed {
        text,
        controls,
        right: _,
    } = split(800.0);

    assert!(controls.left() > text.right(), "{text:?} {controls:?}");
    assert!(controls.top() < text.bottom(), "{text:?} {controls:?}");
}

#[test]
fn on_a_narrow_row_the_controls_go_below_the_text_at_the_left() {
    let Placed {
        text,
        controls,
        right: _,
    } = split(MIN_TEXT_WIDTH + 200.0);

    assert!(controls.top() >= text.bottom(), "{text:?} {controls:?}");
    assert!(
        (controls.left() - text.left()).abs() < 1.0,
        "the controls start where the text does: {controls:?} {text:?}"
    );
}

#[test]
fn controls_wider_than_the_row_wrap_inside_it() {
    let mut harness = harness(260.0, 300.0).build_ui_state(
        |ui, placed: &mut (f32, Vec<Rect>)| {
            placed.0 = ui.max_rect().left();
            let ((), boxes) = split_row(
                ui,
                330.0,
                |ui| {
                    ui.label("Services");
                },
                |ui| {
                    [160.0, 160.0]
                        .map(|width| {
                            ui.allocate_exact_size(vec2(width, 30.0), egui::Sense::click())
                                .0
                        })
                        .to_vec()
                },
            );

            placed.1 = boxes;
        },
        (0.0, Vec::new()),
    );

    harness.run();
    let (left, boxes) = harness.state();

    for placed in boxes {
        assert!(
            placed.left() >= *left - 0.5,
            "{placed:?} starts left of {left}"
        );
    }
}

#[test]
fn a_one_line_title_is_centred_on_a_taller_control() {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        |ui, placed: &mut Placed| {
            let (text, controls) = split_row(
                ui,
                200.0,
                |ui| ui.label("Only when I ask").rect,
                |ui| {
                    ui.allocate_exact_size(vec2(40.0, 34.0), egui::Sense::click())
                        .0
                },
            );

            placed.text = text;
            placed.controls = controls;
        },
        Placed {
            text: Rect::NOTHING,
            controls: Rect::NOTHING,
            right: 0.0,
        },
    );

    harness.run();
    let placed = *harness.state();
    let (text, controls) = (placed.text, placed.controls);
    assert!(
        (text.center().y - controls.center().y).abs() < 1.0,
        "title centre {} control centre {}",
        text.center().y,
        controls.center().y,
    );
}

#[test]
fn a_caption_beside_a_taller_control_is_centred_on_it() {
    let mut harness = harness(600.0, 300.0).build_ui_state(
        |ui, placed: &mut (Rect, Rect)| {
            let ((), (caption, control)) = split_row(
                ui,
                200.0,
                |ui| _ = ui.label("sshd.service"),
                |ui| {
                    let caption = ui.label("At boot").rect;
                    let control = ui
                        .allocate_exact_size(vec2(44.0, 28.0), egui::Sense::click())
                        .0;

                    (caption, control)
                },
            );

            *placed = (caption, control);
        },
        (Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    let (caption, control) = *harness.state();

    assert!(
        (caption.center().y - control.center().y).abs() < 1.0,
        "caption at {caption:?}, control at {control:?}"
    );
}

#[test]
fn a_status_dot_is_centred_on_its_title() {
    for wrap in [false, true] {
        let mut harness = harness(600.0, 200.0).build_ui_state(
            |ui, placed: &mut (Rect, Rect)| {
                // Rows run with the controls' height, taller than a line.
                ui.spacing_mut().interact_size.y = 30.0;
                *placed = title_line(ui, wrap, |ui| {
                    let title = ui.label("PipeWire").rect;
                    let dot = status_dot(ui, "Turns on with Apply", Tone::Accent).rect;
                    (title, dot)
                });
            },
            (Rect::NOTHING, Rect::NOTHING),
        );

        harness.run();
        let (title, dot) = *harness.state();

        assert!(
            title.height() < 25.0,
            "wrap {wrap}: the title took {title:?}"
        );
        assert!(
            (title.center().y - dot.center().y).abs() < 1.0,
            "wrap {wrap}: title at {title:?}, dot at {dot:?}"
        );
    }
}

#[test]
fn a_block_of_text_does_not_take_the_controls_height() {
    let mut heights = Vec::new();
    for control in [18.0, 40.0] {
        let mut harness = harness(200.0, 100.0).build_ui_state(
            move |ui, height: &mut f32| {
                ui.spacing_mut().interact_size.y = control;
                *height = text_block_height(ui, 0);
            },
            0.0,
        );

        harness.run();
        heights.push(*harness.state());
    }

    assert!((heights[0] - heights[1]).abs() < 0.5, "{heights:?}");
}

fn levelled(title: fn(&mut egui::Ui) -> Rect) -> (Rect, Rect) {
    let mut harness = harness(800.0, 300.0).build_ui_state(
        move |ui, placed: &mut (Rect, Rect)| {
            *placed = split_row(
                ui,
                200.0,
                |ui| {
                    _ = title(ui);
                    for line in ["About linux", "6.14.0-4 from core", "It runs right now."] {
                        ui.label(line);
                    }

                    ui.min_rect()
                },
                |ui| {
                    ui.allocate_exact_size(vec2(200.0, 34.0), egui::Sense::click())
                        .0
                },
            );
        },
        (Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    *harness.state()
}

#[test]
fn controls_beside_several_lines_are_centred_on_them_all() {
    let (text, controls) = levelled(|ui| title_line(ui, false, |ui| ui.label("linux").rect));

    assert!(
        (text.center().y - controls.center().y).abs() < 1.0,
        "text at {text:?}, controls at {controls:?}"
    );
}

#[test]
fn controls_beside_text_without_a_title_line_are_centred_on_it_all() {
    let (text, controls) = levelled(|ui| ui.label("linux").rect);

    assert!(
        (text.center().y - controls.center().y).abs() < 1.0,
        "a banner's button sits off its text: {text:?}, {controls:?}"
    );
}

#[test]
fn a_tall_text_beside_short_controls_starts_at_the_top_of_its_row() {
    let mut harness = harness(800.0, 400.0).build_ui_state(
        |ui, placed: &mut (f32, Rect, Rect)| {
            let top = ui.cursor().top();
            let (text, controls) = split_row(
                ui,
                200.0,
                |ui| {
                    title_line(ui, false, |ui| ui.label("linux"));
                    for line in 0_u8..8 {
                        ui.label(format!("Line {line} of the kernel's details"));
                    }

                    ui.min_rect()
                },
                |ui| {
                    ui.allocate_exact_size(vec2(200.0, 34.0), egui::Sense::click())
                        .0
                },
            );

            *placed = (top, text, controls);
        },
        (0.0, Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    let (top, text, controls) = *harness.state();

    assert!(
        text.top() - top < 1.0,
        "the text starts {} below the row's top",
        text.top() - top
    );
    assert!(
        (text.center().y - controls.center().y).abs() < 1.0,
        "text at {text:?}, controls at {controls:?}"
    );
}

/// Add turning into the wider Remove: the frame of the click already ends
/// the new button at the row's right, so nothing jumps for a frame.
#[test]
fn controls_that_grow_end_at_the_right_in_their_first_frame() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, state: &mut (f32, Rect, f32)| {
            let width = state.0;
            let right = ui.max_rect().right();
            let (_, drawn) = split_row(
                ui,
                width,
                |ui| ui.label("babl"),
                |ui| {
                    ui.allocate_exact_size(vec2(width, 34.0), egui::Sense::click())
                        .0
                },
            );

            state.1 = drawn;
            state.2 = right;
        },
        (60.0, Rect::NOTHING, 0.0),
    );

    harness.run();
    harness.state_mut().0 = 90.0;
    harness.step();

    let (_, button, right) = *harness.state();
    assert!(
        (button.right() - right).abs() < 0.5,
        "the grown button ends at {}, the row at {right}",
        button.right()
    );
}
