use egui::Ui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::page_layout;
use crate::motion::PAGE_RISE;
use crate::testing::harness;
use crate::theme;
use crate::widgets::start_page_entry;

#[derive(Clone, Copy, Default)]
struct Shown {
    place: u8,
    start: bool,
    reduced: bool,
}

fn pages(reduced: bool) -> Harness<'static, Shown> {
    let mut harness = harness(400.0, 300.0)
        .with_step_dt(1.0 / 60.0)
        .build_ui_state(
            |root, shown: &mut Shown| {
                theme::set_reduced_motion(root.ctx(), shown.reduced);
                if std::mem::take(&mut shown.start) {
                    start_page_entry(root.ctx());
                }

                page_layout(
                    root,
                    shown.place,
                    &mut (),
                    |(), ui: &mut Ui| {
                        ui.label("Title");
                        true
                    },
                    |(), ui: &mut Ui| _ = ui.label("Row"),
                );
            },
            Shown {
                reduced,
                ..Shown::default()
            },
        );

    harness.run_steps(30);
    harness
}

fn row_top(harness: &Harness<'static, Shown>) -> f32 {
    harness.get_by_label("Row").rect().top()
}

fn first_frame_drop(harness: &mut Harness<'static, Shown>, page: bool) -> f32 {
    let rest = row_top(harness);
    harness.state_mut().place += 1;
    harness.state_mut().start = page;
    harness.step();
    let entering = row_top(harness);
    harness.run_steps(30);
    assert!(
        (row_top(harness) - rest).abs() < 0.5,
        "the row settles back"
    );
    entering - rest
}

#[test]
fn a_new_page_rises_into_place() {
    let mut harness = pages(false);
    let drop = first_frame_drop(&mut harness, true);

    assert!(
        drop > 1.0 && drop <= PAGE_RISE + 0.5,
        "the row starts {drop} lower"
    );
}

#[test]
fn the_first_page_of_the_window_shows_in_place() {
    let mut harness = harness(400.0, 300.0)
        .with_step_dt(1.0 / 60.0)
        .build_ui(|root| {
            page_layout(
                root,
                0_u8,
                &mut (),
                |(), _: &mut Ui| false,
                |(), ui: &mut Ui| {
                    _ = ui.label("Row");
                },
            );
        });

    harness.step();
    let first = harness.get_by_label("Row").rect().top();
    harness.run_steps(30);

    assert!((harness.get_by_label("Row").rect().top() - first).abs() < 0.5);
}

#[test]
fn with_less_motion_a_new_page_shows_in_place() {
    let mut harness = pages(true);

    assert!(first_frame_drop(&mut harness, true).abs() < 0.5);
}

#[test]
fn another_tab_of_the_same_page_shows_in_place() {
    let mut harness = pages(false);

    assert!(first_frame_drop(&mut harness, false).abs() < 0.5);
}

#[test]
fn the_header_stays_while_another_tab_comes_in() {
    let mut harness = pages(false);
    let title = harness.get_by_label("Title").rect();

    // The window spends its second pass on other rows, so this one must
    // come out right in the first.
    harness
        .ctx
        .options_mut(|options| options.max_passes = std::num::NonZeroUsize::MIN);

    harness.state_mut().place += 1;
    harness.step();

    let shown = harness.query_by_label("Title").map(|node| node.rect());
    assert_eq!(shown, Some(title), "the header left for a frame");
}
