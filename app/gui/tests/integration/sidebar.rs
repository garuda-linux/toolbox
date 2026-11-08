mod cut;
mod edge;
mod scroll;
mod strength;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::page::{Group, PageId};
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::testing::lines_in;
use toolbox_gui_kit::tokens::{self, Sizes};

use crate::common::{HEIGHT, SMALLEST, TALL, launch_in, window};

fn app(height: f32, dry_run: bool) -> Harness<'static, App> {
    let setup = Setup {
        dry_run,
        ..Setup::default()
    };

    launch_in(window(height), setup)
}

fn entry_name(harness: &Harness<'_, App>, page: PageId) -> String {
    toolbox_gui::i18n::text(&harness.ctx, page.text_id()).into_owned()
}

fn foot_top(harness: &Harness<'_, App>) -> f32 {
    let settings = entry_name(harness, PageId::Settings);
    harness
        .get_by_role_and_label(Role::Tab, &settings)
        .rect()
        .top()
}

/// Every page's sidebar entry can be reached inside a `height` tall window:
/// above the status bar when a dry run shows it, the scrolled ones above
/// the app's own pages at the foot once scrolled to.
fn every_entry_reachable(height: f32, dry_run: bool) {
    let mut harness = app(height, dry_run);

    let bar_top = harness
        .query_by_label_contains("Test mode: nothing is changed")
        .map_or(height, |bar| bar.rect().top());

    let foot_top = foot_top(&harness);

    for page in PageId::ALL
        .into_iter()
        .filter(|page| *page != PageId::BootRepair)
    {
        let name = entry_name(&harness, page);
        harness
            .get_by_role_and_label(Role::Tab, &name)
            .scroll_to_me();
        harness.run();

        let rect = harness.get_by_role_and_label(Role::Tab, &name).rect();
        assert!(
            rect.bottom() <= bar_top,
            "{page:?} at {rect:?} below {bar_top}"
        );
        if page.group() != Group::App {
            assert!(
                rect.bottom() <= foot_top,
                "{page:?} at {rect:?} hides under the foot at {foot_top}"
            );
        }
    }
}

#[test]
fn a_short_window_keeps_every_page_of_the_sidebar_reachable() {
    every_entry_reachable(HEIGHT, false);
    every_entry_reachable(560.0, false);
}

#[test]
fn the_status_bar_leaves_every_page_of_the_sidebar_reachable() {
    every_entry_reachable(868.0, true);
}

#[test]
fn a_short_window_keeps_the_sidebar_rows_at_their_normal_size() {
    let height = 680.0;
    let harness = app(height, false);
    let row = harness
        .get_by_role_and_label(Role::Tab, "Diagnostics")
        .rect();

    assert!(
        row.height() >= Sizes::DEFAULT.nav_row_height - 0.5,
        "a {height} tall window shrinks a row to {row:?}"
    );
}

#[test]
fn a_collapsed_sidebar_divides_its_named_groups() {
    let mut harness = app(TALL, false);
    harness.get_by_label("Collapse sidebar").click();
    harness.run();
    harness.run();

    let outline = tokens::current(&harness.ctx).outline;
    let install = harness
        .get_by_role_and_label(Role::Tab, "Setup Assistant")
        .rect();

    let dividers: Vec<_> = lines_in(&harness, outline)
        .into_iter()
        .filter(|[left, right]| (left.y - right.y).abs() < 0.5 && right.x <= install.right() + 1.0)
        .collect();

    assert_eq!(dividers.len(), 3, "{dividers:?}");
    let setup_top = install.top();
    assert!(
        dividers.iter().any(|[left, _]| left.y < setup_top),
        "the Install group starts below a line"
    );
}

#[test]
fn a_collapsed_divider_keeps_six_points_to_the_selected_entry_on_each_side() {
    for height in [HEIGHT, TALL] {
        let mut harness = app(height, false);
        harness.get_by_label("Collapse sidebar").click();
        harness.run();
        harness.run();

        let outline = tokens::current(&harness.ctx).outline;
        let entries: Vec<_> = PageId::ALL
            .into_iter()
            .filter(|page| *page != PageId::BootRepair)
            .map(|page| {
                let name = entry_name(&harness, page);
                harness.get_by_role_and_label(Role::Tab, &name).rect()
            })
            .collect();

        // The hairline of a cut edge spans most of an entry, a divider
        // less than half of one.
        let half = entries[0].width() / 2.0;
        let dividers: Vec<f32> = lines_in(&harness, outline)
            .into_iter()
            .filter(|[left, right]| (left.y - right.y).abs() < 0.5 && right.x - left.x < half)
            .map(|[left, _]| left.y)
            .collect();

        assert!(dividers.len() >= 2, "{height}: {dividers:?}");
        for line in dividers {
            let above = entries
                .iter()
                .filter(|entry| entry.bottom() <= line)
                .map(|entry| line - entry.bottom())
                .fold(f32::INFINITY, f32::min);

            let below = entries
                .iter()
                .filter(|entry| entry.top() >= line)
                .map(|entry| entry.top() - line)
                .fold(f32::INFINITY, f32::min);

            assert!(above >= 6.0, "{height}: {above} above the line at {line}");
            assert!(below >= 6.0, "{height}: {below} below the line at {line}");
        }
    }
}

#[test]
fn an_expanded_sidebar_names_its_groups_without_lines() {
    let harness = app(TALL, false);
    let outline = tokens::current(&harness.ctx).outline;
    let sidebar = harness
        .get_by_role_and_label(Role::Tab, "Setup Assistant")
        .rect();

    let lines = lines_in(&harness, outline)
        .into_iter()
        .filter(|[left, right]| (left.y - right.y).abs() < 0.5 && right.x <= sidebar.right() + 1.0)
        .count();

    assert_eq!(lines, 0);
}

fn in_sight_unscrolled(harness: &Harness<'_, App>, page: PageId) -> bool {
    let foot_top = foot_top(harness);

    let name = entry_name(harness, page);
    let rect = harness.get_by_role_and_label(Role::Tab, &name).rect();
    let above_foot = page.group() == Group::App || rect.bottom() <= foot_top;
    above_foot && rect.top() >= 0.0
}

fn faded_within(harness: &Harness<'_, App>, edge: f32) -> bool {
    harness
        .output()
        .shapes
        .iter()
        .any(|clipped| match &clipped.shape {
            egui::Shape::Text(text) => text.pos.x < edge && text.fallback_color.a() < 200,
            // An icon is a rect textured with its glyph.
            egui::Shape::Rect(icon) => {
                icon.brush.is_some()
                    && icon.rect.right() < edge
                    && icon.rect.width() < 40.0
                    && icon.fill.a() < 200
            }
            egui::Shape::Noop
            | egui::Shape::Vec(_)
            | egui::Shape::Circle(_)
            | egui::Shape::Ellipse(_)
            | egui::Shape::LineSegment {
                points: _,
                stroke: _,
            }
            | egui::Shape::Path(_)
            | egui::Shape::Band(_)
            | egui::Shape::Mesh(_)
            | egui::Shape::QuadraticBezier(_)
            | egui::Shape::CubicBezier(_)
            | egui::Shape::Callback(_) => false,
        })
}

fn fade_drawn(harness: &Harness<'_, App>) -> bool {
    faded_within(harness, toolbox_gui_kit::theme::SIDEBAR_WIDTH)
}

#[test]
fn a_window_of_796_shows_every_entry_without_scrolling() {
    let harness = app(796.0, false);

    for page in PageId::ALL
        .into_iter()
        .filter(|page| *page != PageId::BootRepair)
    {
        assert!(
            in_sight_unscrolled(&harness, page),
            "{page:?} needs a scroll"
        );
    }

    assert!(!fade_drawn(&harness), "nothing is cut, so no fade");
}

#[test]
fn a_shorter_window_scrolls_with_a_fade_at_the_cut_edge() {
    let harness = app(600.0, false);

    assert!(
        !in_sight_unscrolled(&harness, PageId::Diagnostics),
        "this test needs a cut list"
    );
    assert!(fade_drawn(&harness));
}

#[test]
fn a_packed_group_heading_has_more_room_above_than_below() {
    let harness = app(HEIGHT, false);
    let rect = |page: PageId| {
        harness
            .get_by_role_and_label(Role::Tab, &entry_name(&harness, page))
            .rect()
    };

    for (heading, above, below) in [
        ("INSTALL", PageId::Home, PageId::Setup),
        ("SYSTEM", PageId::Discover, PageId::Services),
        ("CARE", PageId::Boot, PageId::Maintenance),
    ] {
        let label = harness.get_by_label(heading).rect();
        let over = label.top() - rect(above).bottom();
        let under = rect(below).top() - label.bottom();

        assert!(
            over >= under + 2.0,
            "{heading}: {over} above against {under} below"
        );
    }
}

#[test]
fn a_collapsed_sidebar_fades_its_cut_icons() {
    let harness = launch_in(
        toolbox_gui_kit::testing::harness(SMALLEST.0, SMALLEST.1),
        Setup::default(),
    );

    assert!(
        faded_within(&harness, 80.0),
        "no fade at the rail's cut edge"
    );
}
