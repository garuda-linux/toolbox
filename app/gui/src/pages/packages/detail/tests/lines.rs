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

/// The history's first line starts as far under its fold as the names
/// beside the facts do under their heading.
#[test]
fn the_history_starts_as_far_under_its_heading_as_the_names() {
    let harness = unfolded();

    let names = text_top(&harness, "gegl>=0.4") - text_top(&harness, "Depends on");
    let line = text_top(&harness, "2025-06-01 08:15") - text_top(&harness, "History");
    assert!(
        (line - names).abs() < 0.5,
        "the history's first line {line} under its heading, the names {names}"
    );
}

/// The files' card starts one caption gap under the fold's line, which
/// is as tall as Copy list beside it.
#[test]
fn the_files_card_starts_one_caption_gap_under_copy_list() {
    let harness = unfolded();

    let file = text_top(&harness, "/usr/bin/gimp");
    let card = filled_rects(&harness, tokens::current(&harness.ctx).card)
        .into_iter()
        .filter(|card| card.top() < file && card.bottom() > file)
        .map(|card| card.top())
        .next_back()
        .expect("the files show in a card");

    let copy = rect_of(&harness, Role::Button, "Copy list");
    let gap = tokens::current(&harness.ctx).sizes.caption_gap;
    assert!(
        (card - copy.bottom() - gap).abs() < 0.5,
        "the files' card {} under Copy list",
        card - copy.bottom()
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
