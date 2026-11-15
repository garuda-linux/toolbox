use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::{filled_rects, painted_text};
use toolbox_gui_kit::tokens;

use super::super::super::PackagesPage;
use super::super::super::tests::gimp_details;
use super::{change, showing};
use crate::packages::Read;
use crate::testing::{PageState, rect_of};

fn text_top<S>(harness: &Harness<'_, S>, text: &str) -> f32 {
    let shape = painted_text(harness, text).unwrap_or_else(|| panic!("{text} is painted"));
    shape.pos.y + shape.galley.rect.top()
}

/// gimp with its files unfolded and one line of history, in a window tall
/// enough to paint both.
fn unfolded() -> Harness<'static, PageState<PackagesPage>> {
    let mut details = gimp_details();
    details.history = Read::Known(vec![change(1)]);
    let mut harness = showing(details);
    harness.set_size(vec2(1024.0, 1600.0));
    harness.state_mut().page.unfolded.files = true;
    harness.run();
    harness
}

#[test]
fn every_list_starts_one_gap_under_its_heading() {
    let harness = showing(gimp_details());

    let depends = rect_of(&harness, Role::Heading, "Depends on");
    let gegl = rect_of(&harness, Role::Link, "gegl>=0.4");
    let required = rect_of(&harness, Role::Heading, "Required by");

    // The first None in reading order is Required by's.
    let none = harness
        .get_all_by_role_and_label(Role::Label, "None")
        .next()
        .expect("Required by shows None")
        .rect();

    let names = gegl.top() - depends.bottom();
    let empty = none.top() - required.bottom();
    assert!(
        (names - empty).abs() < 0.5,
        "{names} under Depends on, {empty} under Required by"
    );
}

/// The top of the card that holds the text starting at `top`.
fn card_top<S>(harness: &Harness<'_, S>, top: f32) -> f32 {
    filled_rects(harness, tokens::current(&harness.ctx).card)
        .into_iter()
        .filter(|card| card.top() < top && card.bottom() > top)
        .map(|card| card.top())
        .next_back()
        .expect("the text shows in a card")
}

/// The history's first line starts the card's padding into its card.
#[test]
fn the_history_starts_one_card_padding_into_its_card() {
    let harness = unfolded();

    let shown = crate::dates::pacman_moment(&harness.ctx, "2025-06-01T08:15:00+0200");
    let line = text_top(&harness, &shown.expect("a moment"));
    let into = line - card_top(&harness, line);
    let padding = f32::from(tokens::current(&harness.ctx).sizes.card_padding.top);
    assert!(
        (into - padding).abs() < 0.5,
        "the history's first line {into} into its card, the padding {padding}"
    );
}

/// The filter sits a tight gap under the fold's line, which is as tall as
/// Copy list beside it, and the files' card a control gap under the filter.
#[test]
fn the_files_card_starts_under_the_filter_under_copy_list() {
    let harness = unfolded();

    let file = text_top(&harness, "/usr/bin/gimp");
    let card = card_top(&harness, file);

    let copy = rect_of(&harness, Role::Button, "Copy list");
    let filter = harness.get_by_label("Filter files").rect();
    let sizes = tokens::current(&harness.ctx).sizes;

    // egui puts its item spacing after every widget, on top of the gap.
    let spacing = harness.ctx.global_style().spacing.item_spacing.y;
    assert!(
        (filter.top() - copy.bottom() - sizes.tight_gap - spacing).abs() < 0.5,
        "the filter {} under Copy list",
        filter.top() - copy.bottom()
    );
    assert!(
        (card - filter.bottom() - sizes.control_gap - spacing).abs() < 0.5,
        "the files' card {} under the filter",
        card - filter.bottom()
    );
}

#[test]
fn copy_list_shares_the_middle_of_the_files_fold() {
    let harness = showing(gimp_details());

    let fold = rect_of(&harness, Role::Button, "Files 1");
    let copy = rect_of(&harness, Role::Button, "Copy list");
    assert!(
        (fold.center().y - copy.center().y).abs() < 0.5,
        "the fold's middle at {}, Copy list's at {}",
        fold.center().y,
        copy.center().y
    );
}
