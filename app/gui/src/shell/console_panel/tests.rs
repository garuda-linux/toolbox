use std::cell::Cell;
use std::rc::Rc;

use egui_kittest::kittest::Queryable as _;
use toolbox_plan::StepId;

use super::{step_state, steps};
use crate::console::{StepRow, StepState};

#[test]
fn a_waiting_step_is_told_in_secondary_text() {
    let mut harness = toolbox_gui_kit::testing::harness(300.0, 80.0)
        .build_ui(|ui| step_state(ui, StepState::Waiting, false));

    harness.run();

    let secondary = toolbox_gui_kit::tokens::current(&harness.ctx).secondary;

    assert_eq!(
        toolbox_gui_kit::testing::text_colour(&harness, "Waiting"),
        Some(secondary)
    );
}

#[test]
fn a_long_command_stays_in_its_row_and_shows_whole_on_hover() {
    let command = format!("mkdir -p /run/garuda-toolbox/{}", "long/".repeat(40));
    let rows = vec![StepRow {
        id: StepId::new(1),
        command: command.clone(),
        state: StepState::Skipped,
    }];

    let widest = Rc::new(Cell::new(0.0_f32));
    let seen = Rc::clone(&widest);
    let mut harness = toolbox_gui_kit::testing::harness(500.0, 200.0).build_ui(move |ui| {
        steps(ui, &rows, false);
        seen.set(ui.min_rect().width());
    });

    harness.run();

    assert!(widest.get() <= 500.0, "the steps are {} wide", widest.get());

    harness
        .get_by_label_contains("mkdir -p /run/garuda-toolbox/")
        .hover();
    harness.run();

    assert!(harness.query_all_by_label(&command).count() >= 1);
}

#[test]
fn a_skipped_step_of_a_real_run_had_nothing_to_do() {
    let mut harness = toolbox_gui_kit::testing::harness(300.0, 80.0)
        .build_ui(|ui| step_state(ui, StepState::Skipped, false));

    harness.run();

    let secondary = toolbox_gui_kit::tokens::current(&harness.ctx).secondary;

    assert_eq!(
        toolbox_gui_kit::testing::text_colour(&harness, "Nothing to do"),
        Some(secondary)
    );
}

#[test]
fn a_skipped_step_of_a_test_run_did_not_run() {
    let mut harness = toolbox_gui_kit::testing::harness(300.0, 80.0)
        .build_ui(|ui| step_state(ui, StepState::Skipped, true));

    harness.run();

    assert!(harness.query_by_label("Not run (test mode)").is_some());
}

#[test]
fn a_short_outcome_leaves_no_wide_gap_before_its_command() {
    let rows = vec![StepRow {
        id: StepId::new(1),
        command: "pacman -S firefox".to_owned(),
        state: StepState::Succeeded,
    }];

    let mut harness =
        toolbox_gui_kit::testing::harness(600.0, 200.0).build_ui(move |ui| steps(ui, &rows, false));

    harness.run();

    let done = harness.get_by_label("Done").rect();
    let command = harness.get_by_label("pacman -S firefox").rect();
    let gap = command.left() - done.right();
    let control_gap = toolbox_gui_kit::tokens::current(&harness.ctx)
        .sizes
        .control_gap;

    assert!(
        gap <= control_gap * 2.0,
        "{gap} px between the outcome and its command"
    );
}

fn terminal_box(live: bool) -> egui_kittest::Harness<'static, crate::console::Console> {
    let (sender, _receiver) = crate::command::channel();
    let shown = move |ui: &mut egui::Ui, console: &mut crate::console::Console| {
        super::terminal(ui, console, &sender);
    };

    let builder = if live {
        toolbox_gui_kit::testing::harness(500.0, 200.0)
    } else {
        egui_kittest::Harness::builder()
            .with_size(egui::vec2(500.0, 200.0))
            .with_context_setup(toolbox_gui_kit::install)
    };

    let mut harness = builder.build_ui_state(shown, crate::console::Console::new());
    harness.run();
    harness
}

fn fills(harness: &egui_kittest::Harness<'_, crate::console::Console>) -> (usize, usize) {
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);
    let count = |fill| toolbox_gui_kit::testing::filled_rects(harness, fill).len();
    (count(tokens.terminal_glass), count(tokens.terminal))
}

#[test]
fn over_the_live_blur_the_terminal_box_is_glass() {
    let (glass, solid) = fills(&terminal_box(true));

    assert_eq!(
        (glass > 0, solid),
        (true, 0),
        "{glass} glass, {solid} solid"
    );
}

fn edge(
    harness: &egui_kittest::Harness<'_, crate::console::Console>,
    fill: egui::Color32,
) -> Option<egui::Stroke> {
    harness.output().shapes.iter().find_map(|clipped| {
        if let egui::Shape::Rect(rect) = &clipped.shape
            && rect.fill == fill
        {
            Some(rect.stroke)
        } else {
            None
        }
    })
}

#[test]
fn the_glass_box_keeps_a_hairline_so_it_never_melts_into_the_sheet() {
    let harness = terminal_box(true);
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);

    assert_eq!(
        edge(&harness, tokens.terminal_glass),
        Some(egui::Stroke::new(1.0, tokens.outline))
    );
}

#[test]
fn the_solid_box_needs_no_hairline() {
    let harness = terminal_box(false);
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);

    assert_eq!(edge(&harness, tokens.terminal), Some(egui::Stroke::NONE));
}

#[test]
fn without_the_live_blur_the_terminal_box_is_solid() {
    let (glass, solid) = fills(&terminal_box(false));

    assert_eq!(
        (glass, solid > 0),
        (0, true),
        "{glass} glass, {solid} solid"
    );
}

fn ringed(harness: &egui_kittest::Harness<'_, crate::console::Console>) -> bool {
    let tokens = toolbox_gui_kit::tokens::current(&harness.ctx);
    let frame = toolbox_gui_kit::testing::filled_rect(harness, tokens.terminal).expect("the box");
    toolbox_gui_kit::testing::rims_in(harness, tokens.accent_ring)
        .iter()
        .any(|(rect, _)| {
            (rect.min - frame.min).length() < 0.5 && (rect.max - frame.max).length() < 0.5
        })
}

/// Keys typed into the terminal would bring a ring back the next click
/// hides, so its cursor alone shows the focus.
#[test]
fn a_focused_terminal_never_rings_its_box() {
    use egui_kittest::kittest::Queryable as _;

    let mut harness = terminal_box(false);
    harness.get_by_role(egui::accesskit::Role::Terminal).focus();
    harness.run();
    harness.key_press(egui::Key::Tab);
    harness.run();

    assert!(
        harness
            .get_by_role(egui::accesskit::Role::Terminal)
            .is_focused()
    );
    assert!(!ringed(&harness));
}

#[test]
fn the_idle_hint_sits_one_cell_after_the_cursor_on_its_row() {
    let terminal = egui::Rect::from_min_size(egui::pos2(40.0, 30.0), egui::vec2(500.0, 100.0));
    let mut harness = toolbox_gui_kit::testing::harness(600.0, 200.0)
        .build_ui(move |ui| super::idle::idle_hint(ui, terminal, false));

    harness.run();

    let font = egui::TextStyle::Monospace.resolve(&harness.ctx.global_style());
    let cell = harness
        .ctx
        .fonts_mut(|fonts| egui::vec2(fonts.glyph_width(&font, 'M'), fonts.row_height(&font)));

    let hint = harness
        .get_by_label("The output of the next apply shows here.")
        .rect();

    assert!(
        (hint.left() - cell.x.mul_add(2.0, terminal.left())).abs() < 0.5,
        "the hint starts at {}, the cell is {cell:?}",
        hint.left()
    );
    assert!(
        (hint.center().y - (terminal.top() + cell.y / 2.0)).abs() < 0.5,
        "the hint is centred at {}, the cell is {cell:?}",
        hint.center().y
    );
}

/// The rects the sheet fully out in a 600 by 400 room painted, with the
/// clip of each, the sheet's top and the outline colour.
fn sheet_rects() -> (
    Vec<(egui::Rect, egui::epaint::RectShape)>,
    f32,
    egui::Color32,
) {
    let room = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(600.0, 400.0));
    let place = super::Place { room, out: 1.0 };
    let frame = egui::Frame::new().corner_radius(8);
    let mut harness = toolbox_gui_kit::testing::harness(600.0, 400.0).build_ui(move |ui| {
        _ = super::sheet::show(ui.ctx(), place, "Resize", frame, |_| {});
    });

    harness.run();
    let top = super::cover(&harness.ctx).expect("the sheet shows").top();
    let rects = harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let egui::Shape::Rect(rect) = &clipped.shape {
                Some((clipped.clip_rect, rect.clone()))
            } else {
                None
            }
        })
        .collect();

    let outline = toolbox_gui_kit::tokens::current(&harness.ctx).outline;
    (rects, top, outline)
}

#[test]
fn the_sheet_has_a_hairline_on_its_top_edge() {
    let (rects, top, outline) = sheet_rects();
    let edged = rects.iter().any(|(clip, rect)| {
        rect.stroke.color == outline
            && rect.stroke.width >= 1.0
            && (rect.rect.top() - top).abs() < 0.5
            && clip.top() <= top + 0.5
            && clip.bottom() > top
    });

    assert!(edged, "no hairline at the sheet's top {top}: {rects:?}");
}

#[test]
fn the_sheet_casts_its_shadow_up_over_the_page_only() {
    let (rects, top, _) = sheet_rects();
    let shadows: Vec<_> = rects
        .iter()
        .filter(|(_, rect)| rect.blur_width > 0.0)
        .collect();

    assert!(
        shadows
            .iter()
            .any(|(clip, rect)| rect.rect.top() < top && clip.bottom() <= top + 0.5),
        "no shadow above the sheet's top {top}: {rects:?}"
    );
    assert!(
        shadows.iter().all(|(clip, _)| clip.bottom() <= top + 0.5),
        "a shadow reaches under the sheet's glass: {shadows:?}"
    );
}

fn has_block_cursor(harness: &egui_kittest::Harness<'_, crate::console::Console>) -> bool {
    let text = toolbox_gui_kit::palette::current(&harness.ctx).text;
    !toolbox_gui_kit::testing::filled_rects(harness, text).is_empty()
}

#[test]
fn an_idle_terminal_shows_no_cursor() {
    assert!(!has_block_cursor(&terminal_box(false)));
}

#[test]
fn a_running_plan_shows_the_terminal_cursor() {
    let mut harness = terminal_box(false);
    harness.state_mut().take(crate::apply::Report::PlanStarted {
        plan: crate::apply::PlanId::new(1),
        steps: vec![(StepId::new(1), vec!["pacman -S firefox".to_owned()])],
    });
    harness.run();

    assert!(has_block_cursor(&harness));
}

/// A long plan runs its later steps below the list's fold, where the
/// running one would hide.
#[test]
fn the_running_step_scrolls_into_view() {
    let shown = |running: u32| {
        let rows: Vec<StepRow> = (1..=12_u32)
            .map(|number| StepRow {
                id: StepId::new(number),
                command: format!("step {number}"),
                state: match number.cmp(&running) {
                    std::cmp::Ordering::Less => StepState::Succeeded,
                    std::cmp::Ordering::Equal => StepState::Running,
                    std::cmp::Ordering::Greater => StepState::Waiting,
                },
            })
            .collect();

        let mut harness = toolbox_gui_kit::testing::harness(600.0, 400.0)
            .build_ui(move |ui| steps(ui, &rows, false));

        harness.run();
        harness
    };

    let first = shown(1).get_by_label("step 1").rect();
    let running = shown(11).get_by_label("step 11").rect();
    assert!(
        running.bottom() <= first.top() + super::STEPS_HEIGHT + 1.0,
        "the running step ends at {}, the list shows {} px from {}",
        running.bottom(),
        super::STEPS_HEIGHT,
        first.top()
    );
}
