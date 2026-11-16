//! The Console as a sheet over the page: three quarters of the window
//! tall, the page full size and usable above it, the grip on its bar.

mod under;

use egui::accesskit::Role;
use egui::{Key, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::{App, Setup};

use crate::common::{drag, launch_in, open_console, pane, window};

const HEIGHT: f32 = 1200.0;

fn app_with(setup: Setup) -> Harness<'static, App> {
    launch_in(window(HEIGHT).with_max_steps(120), setup)
}

fn app() -> Harness<'static, App> {
    app_with(Setup::default())
}

fn grip(harness: &Harness<'static, App>) -> Pos2 {
    harness.get_by_label("Resize the Console").rect().center()
}

#[test]
fn the_console_opens_over_three_quarters_of_the_page() {
    let mut harness = app();
    let page = pane(&harness, "Page").expect("page");
    let sheet = open_console(&mut harness);

    let top = page.height().mul_add(-0.75, page.bottom());
    assert!(
        (sheet.top() - top).abs() < 40.0,
        "sheet {sheet:?} against a top of {top} in {page:?}"
    );

    // The pane ends with the terminal, above the sheet's bottom padding.
    assert!((sheet.bottom() - page.bottom()).abs() < 32.0, "{sheet:?}");
}

#[test]
fn the_page_keeps_its_size_under_the_console() {
    let mut harness = app();
    let before = pane(&harness, "Page").expect("page");
    open_console(&mut harness);
    let after = pane(&harness, "Page").expect("page");

    assert_eq!(before, after);
}

#[test]
fn the_page_above_the_console_still_takes_clicks() {
    let mut harness = app();
    let sheet = open_console(&mut harness);
    let tile = harness
        .get_all_by_role(Role::Link)
        .find(|node| {
            let rect = node.rect();
            rect.left() > 260.0
                && rect.bottom() < sheet.top()
                && node
                    .accesskit_node()
                    .label()
                    .is_some_and(|label| label.starts_with("Updates"))
        })
        .expect("the Updates tile above the sheet");

    tile.click();
    harness.run();

    assert!(
        pane(&harness, "Console").is_some(),
        "a click on the page leaves the Console open"
    );
    assert!(
        harness
            .query_by_label_contains("Keep the system in a good state")
            .is_some(),
        "the click reached the page"
    );
}

#[test]
fn the_grip_bar_drags_the_sheet_taller() {
    let mut harness = app();
    let sheet = open_console(&mut harness);
    let from = grip(&harness);
    assert!(
        from.y > sheet.top() - 24.0 && from.y < sheet.top() + 24.0,
        "the grip sits on the sheet's top: {from:?} {sheet:?}"
    );

    drag(&mut harness, from, Pos2::new(from.x, from.y - 120.0));
    harness.run();
    let taller = pane(&harness, "Console").expect("open");

    assert!(taller.top() < sheet.top() - 80.0, "{taller:?} {sheet:?}");
}

fn focus_grip(harness: &mut Harness<'static, App>) {
    harness
        .get_by_role_and_label(Role::Splitter, "Resize the Console")
        .focus();
    harness.run();
}

fn press(harness: &mut Harness<'static, App>, key: Key) -> egui::Rect {
    harness.key_press(key);
    harness.run();
    pane(harness, "Console").expect("open")
}

#[test]
fn up_and_down_on_the_focused_grip_resize_the_sheet() {
    let mut harness = app();
    let sheet = open_console(&mut harness);
    focus_grip(&mut harness);

    let taller = press(&mut harness, Key::ArrowUp);
    assert!(taller.top() < sheet.top() - 8.0, "{taller:?} {sheet:?}");

    let back = press(&mut harness, Key::ArrowDown);
    assert!((back.top() - sheet.top()).abs() < 1.0, "{back:?} {sheet:?}");
    assert!(
        harness
            .get_by_role_and_label(Role::Splitter, "Resize the Console")
            .is_focused(),
        "the arrows kept the focus on the grip"
    );
}

#[test]
fn home_and_end_take_the_sheet_to_its_most_and_least_height() {
    let mut harness = app();
    let sheet = open_console(&mut harness);
    focus_grip(&mut harness);

    let most = press(&mut harness, Key::Home);
    let least = press(&mut harness, Key::End);

    assert!(most.top() < sheet.top() - 80.0, "{most:?} {sheet:?}");
    assert!(least.top() > sheet.top() + 80.0, "{least:?} {sheet:?}");
    assert!(pane(&harness, "Console").is_some(), "End leaves it open");
}

#[test]
fn the_focused_grip_shows_its_ring_and_tells_its_height() {
    let mut harness = app();
    open_console(&mut harness);
    focus_grip(&mut harness);

    let grip = harness.get_by_role_and_label(Role::Splitter, "Resize the Console");
    let ring = toolbox_gui_kit::tokens::current(&harness.ctx).accent_ring;
    let rect = grip.rect();
    let share = grip.accesskit_node().numeric_value();

    assert!(
        toolbox_gui_kit::testing::rims_in(&harness, ring)
            .iter()
            .any(|(rim, _)| rim.intersects(rect)),
        "no ring on the grip at {rect:?}"
    );
    assert!(
        share.is_some_and(|share| share > 50.0_f64 && share <= 100.0_f64),
        "{share:?}"
    );
}

#[test]
fn escape_closes_the_console() {
    let mut harness = app();
    open_console(&mut harness);

    harness.key_press(Key::Escape);
    harness.run();

    assert!(pane(&harness, "Console").is_none());
}

#[test]
fn escape_with_the_changes_open_closes_them_and_keeps_the_console() {
    let mut harness = crate::changes::app_with(&["firefox"]);
    open_console(&mut harness);
    harness.get_by_label("Review").click();
    harness.run();
    assert!(harness.query_by_label("Queued changes").is_some());

    harness.key_press(Key::Escape);
    harness.run();

    assert!(harness.query_by_label("Queued changes").is_none());
    assert!(pane(&harness, "Console").is_some());
}

#[test]
fn the_sheet_blurs_the_page_under_it() {
    let mut harness = app();
    let sheet = open_console(&mut harness);

    let blurred = toolbox_gui_kit::testing::blurred_rects(&harness);
    assert!(
        blurred
            .iter()
            .any(|rect| rect.top() <= sheet.top() + 1.0 && rect.width() >= sheet.width() - 1.0),
        "no blur under the sheet at {sheet:?}: {blurred:?}"
    );
}

#[test]
fn the_terminal_reaches_the_sheets_padding_and_its_hint_sits_in_the_header() {
    let hint = "Ctrl+Tab leaves the terminal.";
    let mut harness = app();
    open_console(&mut harness);
    harness
        .get_by_role_and_label(Role::Terminal, "Console output")
        .click();
    harness.run();

    let sizes = toolbox_gui_kit::tokens::current(&harness.ctx).sizes;
    let spacing = harness.ctx.global_style().spacing.item_spacing.y;
    let terminal = harness
        .get_by_role_and_label(Role::Terminal, "Console output")
        .rect();

    let frame = terminal.bottom() + f32::from(sizes.terminal_padding.bottom);
    let page = pane(&harness, "Page").expect("page");
    let band = page.bottom() - frame;
    let room = f32::from(sizes.sheet_padding.bottom) + spacing;
    assert!(
        band <= room + 1.0,
        "{band} below the terminal's frame at {frame}, the sheet's padding is {room}"
    );

    let shown = toolbox_gui_kit::testing::painted_text(&harness, hint)
        .expect("the hint is gone while the terminal has the focus")
        .visual_bounding_rect();

    assert!(
        shown.bottom() <= terminal.top(),
        "the hint at {shown:?} is not above the terminal at {terminal:?}"
    );
}

/// A read in progress shows in the sidebar right above Settings, where
/// neither the page nor the sheet reaches.
#[test]
fn the_reading_note_sits_in_the_sidebar_above_settings() {
    let (mut harness, _runtime, _answer, _reads) = crate::system::app();
    crate::system::step_until(&mut harness, "the reading notice", |harness| {
        harness.query_all_by_label("Reading the system…").count() == 1
    });
    let console = open_console(&mut harness);

    let note = harness.get_by_label("Reading the system…").rect();
    let sidebar = pane(&harness, "Pages").expect("sidebar");
    let settings = harness.get_by_role_and_label(Role::Tab, "Settings").rect();

    assert!(
        sidebar.contains_rect(note),
        "the note at {note:?} is off the sidebar at {sidebar:?}"
    );
    assert!(note.bottom() <= settings.top(), "the note covers Settings");
    assert!(
        settings.top() - note.bottom() < 20.0,
        "the note is far from Settings"
    );
    assert!(note.right() < console.left(), "the Console covers the note");
    harness.snapshot("shell/sidebar_reading");

    harness.get_by_label("Collapse sidebar").click();
    harness.run();
    let rail = pane(&harness, "Pages").expect("sidebar");
    let dots = harness.get_by_label("Reading the system…").rect();
    assert!(
        rail.contains_rect(dots),
        "the dots at {dots:?} are off the rail at {rail:?}"
    );
    harness.snapshot("shell/sidebar_reading_collapsed");
}
