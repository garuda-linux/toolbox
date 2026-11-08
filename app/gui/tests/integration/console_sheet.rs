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
    assert!((sheet.bottom() - page.bottom()).abs() < 24.0, "{sheet:?}");
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
                    .is_some_and(|label| label.contains("Setup Assistant"))
        })
        .expect("the Setup Assistant tile above the sheet");

    tile.click();
    harness.run();

    assert!(
        pane(&harness, "Console").is_some(),
        "a click on the page leaves the Console open"
    );
    assert!(
        harness
            .query_by_label_contains("Select the software")
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

#[test]
fn escape_closes_the_console() {
    let mut harness = app();
    open_console(&mut harness);

    harness.key_press(Key::Escape);
    harness.run();

    assert!(pane(&harness, "Console").is_none());
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
