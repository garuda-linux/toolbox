//! The pictures of an AUR build: its row in the changes and the review.

use egui::Theme;
use image::RgbaImage;
use toolbox_gui::apply::Report;
use toolbox_plan::StepId;
use toolbox_plan::protocol::Event;

use super::named_pane;
use crate::changes::build_pictures::{built_popover_on, console_review_on};
use crate::common::shared_runtime;
use crate::look::look_for;

/// What the review printed for octopi-git in the Garuda Lima VM, its
/// first lines.
const REVIEW_OUTPUT: &str = include_str!("../fixtures/lima/aur-review-octopi-git.txt");

pub(super) fn build(theme: Theme) -> RgbaImage {
    let look = look_for(shared_runtime(), theme);
    let mut harness = built_popover_on(theme, shared_runtime(), look);

    named_pane(&mut harness, "Changes to apply")
}

pub(super) fn review(theme: Theme) -> RgbaImage {
    let look = look_for(shared_runtime(), theme);
    let (mut harness, reports) = console_review_on(theme, look);

    reports.send(Report::Step(Event::Output {
        step: StepId::new(1),
        data: REVIEW_OUTPUT.replace('\n', "\r\n"),
    }));
    harness.run();

    // Without the focus the terminal's cursor is hidden, as in the other Console pictures.
    crate::console::unfocus(&mut harness);
    named_pane(&mut harness, "Console")
}
