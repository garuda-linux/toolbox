//! The sidebar's list once it scrolls: its cut edges and where it stops.

use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::page::{Group, PageId};
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::testing::lines_in;
use toolbox_gui_kit::tokens;

use super::cut::app_on;
use super::{entry_name, foot_top};
use crate::common::{HEIGHT, SMALLEST, launch_in, window};

const HEADINGS: [&str; 3] = ["INSTALL", "SYSTEM", "CARE"];

fn first_below(harness: &Harness<'_, App>, top: f32) -> Rect {
    let entries = PageId::ALL
        .into_iter()
        .filter(|page| page.group() != Group::App && *page != PageId::BootRepair)
        .map(|page| {
            harness
                .get_by_role_and_label(Role::Tab, &entry_name(harness, page))
                .rect()
        });

    let headings = HEADINGS
        .into_iter()
        .map(|heading| harness.get_by_label(heading).rect());

    entries
        .chain(headings)
        .filter(|rect| rect.top() >= top - 0.5)
        .min_by(|one, other| one.top().total_cmp(&other.top()))
        .expect("an item below the edge")
}

fn text_on(harness: &Harness<'_, App>, item: Rect) -> Vec<egui::epaint::TextShape> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let egui::Shape::Text(text) = &clipped.shape
                && Rect::from_min_size(text.pos, text.galley.size()).intersects(item)
            {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect()
}

fn ink_top(harness: &Harness<'_, App>, item: Rect) -> f32 {
    text_on(harness, item)
        .iter()
        .map(|text| text.pos.y)
        .fold(f32::INFINITY, f32::min)
}

#[test]
fn a_list_scrolled_to_a_group_keeps_its_room_above_the_heading() {
    let at_rest = app_on(HEIGHT, PageId::Home);
    let scrolled = app_on(HEIGHT, PageId::Maintenance);
    let top = list_top(&scrolled).expect("the list scrolled to Maintenance");
    let home = at_rest
        .get_by_role_and_label(Role::Tab, &entry_name(&at_rest, PageId::Home))
        .rect();

    // What shows of an entry is its text, its row is see-through.
    let room = ink_top(&at_rest, home) - top;
    let first = first_below(&scrolled, top);
    let above = ink_top(&scrolled, first) - top;
    assert!(
        above >= room - 0.5,
        "the first item {first:?} shows {above} below {top}, Home at rest {room}"
    );
}

#[test]
fn the_item_at_a_scrolled_lists_top_edge_is_faded() {
    let scrolled = app_on(HEIGHT, PageId::Maintenance);
    let top = list_top(&scrolled).expect("the list scrolled to Maintenance");
    let first = first_below(&scrolled, top);

    let faded = text_on(&scrolled, first)
        .iter()
        .any(|text| text.fallback_color.a() < 250);

    assert!(faded, "the item {first:?} at the edge {top} is not faded");
}

/// Where the list's top edge lies once it scrolled, by its hairline.
fn list_top(harness: &Harness<'_, App>) -> Option<f32> {
    let sidebar = toolbox_gui_kit::theme::SIDEBAR_WIDTH;
    let edge = tokens::current(&harness.ctx).sidebar_edge;
    lines_in(harness, edge)
        .iter()
        .find(|[start, end]| {
            // The divider at the sidebar's side runs down, the cut edge across.
            (start.y - end.y).abs() < 0.5 && end.x <= sidebar + 1.0 && start.y < 200.0
        })
        .map(|[start, _]| start.y)
}

#[test]
fn the_list_scrolled_to_the_open_page_keeps_its_room_at_the_top() {
    let at_rest = app_on(600.0, PageId::Home);
    let scrolled = app_on(600.0, PageId::Diagnostics);
    let top = list_top(&scrolled).expect("the list scrolled to Diagnostics");
    let entry = |harness: &Harness<'_, App>, page: PageId| {
        harness
            .get_by_role_and_label(Role::Tab, &entry_name(harness, page))
            .rect()
    };

    let room = entry(&at_rest, PageId::Home).top() - top;
    let listed = PageId::ALL
        .into_iter()
        .filter(|page| page.group() != Group::App && *page != PageId::BootRepair);

    let first = listed
        .clone()
        .map(|page| entry(&scrolled, page))
        .filter(|rect| rect.top() >= top - 0.5)
        .min_by(|one, other| one.top().total_cmp(&other.top()))
        .expect("an entry below the edge");

    for page in listed {
        let rect = entry(&scrolled, page);
        assert!(
            // The gap between two entries may lie under the edge.
            rect.bottom() <= top + 1.5 || rect.top() >= top - 0.5,
            "{page:?} at {rect:?} is cut at {top}"
        );
    }

    assert!(
        first.top() - top >= room - 0.5,
        "the first entry {first:?} sits closer to {top} than Home does at rest"
    );
}

#[test]
fn an_expanded_sidebar_marks_its_cut_edge_with_a_hairline() {
    let harness = launch_in(window(600.0), Setup::default());

    let sidebar = toolbox_gui_kit::theme::SIDEBAR_WIDTH;
    let edge = tokens::current(&harness.ctx).sidebar_edge;
    let marked = lines_in(&harness, edge)
        .iter()
        .any(|[start, end]| end.x <= sidebar + 1.0 && end.x - start.x > sidebar / 2.0);

    assert!(marked, "no hairline at the sidebar's cut edge");
}

#[test]
fn a_collapsed_sidebar_in_a_short_window_marks_its_cut_edge() {
    // Between icons the fade alone is too faint to tell the list goes on.
    let harness = launch_in(
        toolbox_gui_kit::testing::harness(SMALLEST.0, SMALLEST.1),
        Setup::default(),
    );

    let outline = tokens::current(&harness.ctx).sidebar_edge;
    let edge = 80.0;
    let marked = lines_in(&harness, outline)
        .iter()
        .any(|[start, end]| end.x <= edge && end.x - start.x > 20.0 && start.y > 300.0);

    assert!(marked, "no hairline where the collapsed list is cut");
}

#[test]
fn the_current_page_scrolls_into_the_sidebar() {
    let harness = app_on(600.0, PageId::Diagnostics);
    let foot_top = foot_top(&harness);

    let name = entry_name(&harness, PageId::Diagnostics);
    let rect = harness.get_by_role_and_label(Role::Tab, &name).rect();
    assert!(
        rect.bottom() <= foot_top + 0.5,
        "the open page's entry {rect:?} lies under the foot at {foot_top}"
    );
}

#[test]
fn a_heading_keeps_room_above_even_when_the_list_scrolls() {
    let harness = app_on(600.0, PageId::Home);
    let home = harness
        .get_by_role_and_label(Role::Tab, &entry_name(&harness, PageId::Home))
        .rect();

    let heading = harness.get_by_label("INSTALL").rect();
    assert!(
        heading.top() - home.bottom() >= 4.0,
        "the heading {heading:?} sticks to the entry above {home:?}"
    );
}
