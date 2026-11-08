//! What slides and what jumps: the pointer's changes move, a shortcut's
//! land at once.

use egui::accesskit::Role;
use egui::{Key, Modifiers, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::theme;

use crate::common::{HEIGHT, drag, open_console, palette, pane, window};

/// About the sheet's height when first opened.
const SHEET: f32 = 300.0;
/// The collapsed sidebar's width, its margins included.
const COLLAPSED: f32 = toolbox_gui::testing::COLLAPSED_SIDEBAR;

/// The app at sixty frames a second with egui's own motion on, which the
/// harness turns off.
fn app() -> Harness<'static, App> {
    let mut harness = window(HEIGHT)
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build_eframe(|cc| App::new(cc, Setup::default()));

    harness.ctx.all_styles_mut(|style| {
        style.animation_time = theme::ANIMATION_TIME;
    });
    harness.run();
    harness
}

fn a_moment(harness: &mut Harness<'static, App>) {
    for _ in 0..3_u8 {
        harness.step();
    }
}

#[test]
fn ctrl_j_opens_and_closes_the_console_at_once() {
    let mut harness = app();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    a_moment(&mut harness);
    let open = pane(&harness, "Console").expect("open");
    assert!(open.height() >= SHEET, "{open:?}");
    assert!(open.bottom() <= HEIGHT + 0.5, "in place: {open:?}");

    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    a_moment(&mut harness);
    assert!(pane(&harness, "Console").is_none());
}

#[test]
fn a_click_slides_the_console_away() {
    let mut harness = app();
    let open = open_console(&mut harness);

    harness
        .get_by_role_and_label(Role::Button, "Hide the Console")
        .click();
    a_moment(&mut harness);

    // It leaves slowly at first, so a few frames in it has only begun.
    let sliding = pane(&harness, "Console").expect("still sliding");
    assert!(sliding.top() > open.top(), "{sliding:?} {open:?}");

    harness.run();
    assert!(pane(&harness, "Console").is_none());
}

#[test]
fn with_less_motion_a_click_closes_the_console_at_once() {
    let mut harness = app();
    theme::set_reduced_motion(&harness.ctx, true);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    harness.run();

    harness
        .get_by_role_and_label(Role::Button, "Hide the Console")
        .click();
    a_moment(&mut harness);

    assert!(pane(&harness, "Console").is_none());
}

#[test]
fn ctrl_b_collapses_the_sidebar_at_once() {
    let mut harness = app();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::B);
    a_moment(&mut harness);

    let sidebar = pane(&harness, "Pages").expect("sidebar");
    assert!(sidebar.width() <= COLLAPSED + 0.5, "{sidebar:?}");
}

#[test]
fn a_click_slides_the_sidebar_shut() {
    let mut harness = app();

    harness
        .get_by_role_and_label(Role::Button, "Collapse sidebar")
        .click();
    a_moment(&mut harness);
    let sliding = pane(&harness, "Pages").expect("sidebar");
    assert!(sliding.width() > COLLAPSED + 0.5, "{sliding:?}");

    harness.run();
    let shut = pane(&harness, "Pages").expect("sidebar");
    assert!(shut.width() <= COLLAPSED + 0.5, "{shut:?}");
}

/// How far the Console is out, from its top against `open`, its top when
/// fully out. None once it is gone.
fn sheet_out(harness: &Harness<'static, App>, open: f32) -> Option<f32> {
    pane(harness, "Console").map(|rect| (HEIGHT - rect.top()) / (HEIGHT - open))
}

fn moving(harness: &mut Harness<'static, App>, open: f32) -> Vec<f32> {
    let mut shares = Vec::new();
    for _ in 0..60_u8 {
        harness.step();
        match sheet_out(harness, open) {
            Some(share) if share > 0.001 && share < 0.999 => shares.push(share),
            _ => {}
        }
    }

    shares
}

fn open_from_the_palette(harness: &mut Harness<'static, App>) {
    palette(harness, "console");

    // The row's name ends in its shortcut, and it is drawn over the page.
    harness
        .query_all_by_label_contains("Toggle console")
        .last()
        .expect("the palette's row")
        .click();
}

#[test]
fn enter_in_the_palette_opens_the_console_at_once() {
    let mut harness = app();
    palette(&mut harness, "console");

    harness.key_press(Key::Enter);
    a_moment(&mut harness);

    let open = pane(&harness, "Console").expect("open");
    assert!(open.height() >= SHEET, "{open:?}");
    assert!(open.bottom() <= HEIGHT + 0.5, "in place: {open:?}");
}

fn open_top(harness: &mut Harness<'static, App>) -> f32 {
    let top = open_console(harness).top();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    harness.run();
    top
}

#[test]
fn the_console_slides_open_fast_first_and_ends_fully_out() {
    let mut harness = app();
    let top = open_top(&mut harness);

    open_from_the_palette(&mut harness);
    let shares = moving(&mut harness, top);

    assert!(shares.len() >= 3, "it moves: {shares:?}");
    let middle = shares[shares.len() / 2];
    assert!(middle > 0.6, "eases out: {shares:?}");
    let rest = sheet_out(&harness, top).expect("open");
    assert!((rest - 1.0).abs() < 0.01, "{rest}");
}

#[test]
fn the_console_slides_away_slow_first_and_ends_gone() {
    let mut harness = app();
    let top = open_console(&mut harness).top();

    harness
        .get_by_role_and_label(Role::Button, "Hide the Console")
        .click();
    let shares = moving(&mut harness, top);

    assert!(shares.len() >= 3, "it moves: {shares:?}");
    let middle = shares[shares.len() / 2];
    assert!(
        middle > 0.6,
        "eases in, above the straight line: {shares:?}"
    );
    assert!(pane(&harness, "Console").is_none());
}

#[test]
fn with_less_motion_the_console_opens_at_once() {
    let mut harness = app();
    theme::set_reduced_motion(&harness.ctx, true);

    open_from_the_palette(&mut harness);
    a_moment(&mut harness);

    let open = pane(&harness, "Console").expect("open");
    assert!(open.height() >= SHEET, "{open:?}");
    assert!(open.bottom() <= HEIGHT + 0.5, "in place: {open:?}");
}

#[test]
fn dragging_the_grip_down_closes_the_console() {
    let mut harness = app();
    let open = open_console(&mut harness);

    // The panel's edge lies above the sheet's top margin.
    let edge = Pos2::new(open.center().x, open.top() - 10.0);

    drag(&mut harness, edge, Pos2::new(edge.x, HEIGHT - 2.0));
    harness.run();
    assert!(pane(&harness, "Console").is_none(), "closed");

    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    harness.run();
    assert!(
        pane(&harness, "Console").is_some(),
        "the next Ctrl+J opens it"
    );
}

#[test]
fn the_console_opens_again_at_the_height_it_was_dragged_to() {
    let mut harness = app();
    let open = open_console(&mut harness);
    let edge = Pos2::new(open.center().x, open.top() - 10.0);

    drag(&mut harness, edge, Pos2::new(edge.x, edge.y - 100.0));
    harness.run();
    let taller = pane(&harness, "Console").expect("open").top();
    assert!(taller < open.top() - 50.0, "{taller} {open:?}");

    harness
        .get_by_role_and_label(Role::Button, "Hide the Console")
        .click();
    harness.run();
    open_from_the_palette(&mut harness);
    let shares = moving(&mut harness, taller);

    // It slides to the dragged height, not to a guess it jumps from.
    let last = shares.last().copied().unwrap_or_default();
    assert!(last > 0.95, "{shares:?}");
    let again = pane(&harness, "Console").expect("open").top();
    assert!((again - taller).abs() < 1.0, "{again} {taller}");
}
