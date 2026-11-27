mod cut;
mod edge;
mod foot;
mod head;
mod highlight;
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

fn sidebar_pages() -> impl Iterator<Item = PageId> {
    PageId::ALL
        .into_iter()
        .filter(|page| *page != PageId::BootRepair)
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

    for page in sidebar_pages() {
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

    let outline = tokens::current(&harness.ctx).sidebar_edge;
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

        let outline = tokens::current(&harness.ctx).sidebar_edge;
        let rows: Vec<(Group, _)> = sidebar_pages()
            .map(|page| {
                let name = entry_name(&harness, page);
                let rect = harness.get_by_role_and_label(Role::Tab, &name).rect();
                (page.group(), rect)
            })
            .collect();

        let entries: Vec<_> = rows.iter().map(|(_, rect)| *rect).collect();

        // The cut hairline spans most of an entry, a divider less than half.
        // The list's cut hairline sits above the foot, which has no divider,
        // so a line above a foot entry is not a group divider.
        let foot_tops: Vec<f32> = rows
            .iter()
            .filter(|(group, _)| *group == Group::App)
            .map(|(_, rect)| rect.top())
            .collect();

        let foot_next = |line: f32| {
            entries
                .iter()
                .filter(|entry| entry.top() >= line)
                .min_by(|one, other| one.top().total_cmp(&other.top()))
                .is_some_and(|entry| foot_tops.contains(&entry.top()))
        };

        let half = entries[0].width() / 2.0;
        let dividers: Vec<f32> = lines_in(&harness, outline)
            .into_iter()
            .filter(|[left, right]| (left.y - right.y).abs() < 0.5 && right.x - left.x < half)
            .map(|[left, _]| left.y)
            .filter(|line| !foot_next(*line))
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
    // At normal spacing nothing is cut, so only a divider could draw a line.
    let harness = app(964.0, false);
    let outline = tokens::current(&harness.ctx).sidebar_edge;
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

fn assert_unscrolled_and_uncut(harness: &Harness<'_, App>) {
    for page in sidebar_pages() {
        assert!(
            in_sight_unscrolled(harness, page),
            "{page:?} needs a scroll"
        );
    }

    assert!(!fade_drawn(harness), "nothing is cut, so no fade");
}

fn packed_heading_gaps(harness: &Harness<'_, App>) -> [(&'static str, f32, f32); 3] {
    let rect = |page: PageId| {
        harness
            .get_by_role_and_label(Role::Tab, &entry_name(harness, page))
            .rect()
    };

    [
        ("Install", PageId::Home, PageId::Setup),
        ("System", PageId::Discover, PageId::Services),
        ("Care", PageId::Boot, PageId::Maintenance),
    ]
    .map(|(heading, above, below)| {
        let label = harness.get_by_role_and_label(Role::Heading, heading).rect();
        let over = label.top() - rect(above).bottom();
        let under = rect(below).top() - label.bottom();
        (heading, over, under)
    })
}

/// 964 points show every entry in its normal spacing (KDE pitch rows).
#[test]
fn a_window_of_964_shows_every_entry_unpacked_without_scrolling() {
    let harness = app(964.0, false);
    let rect = |page: PageId| {
        harness
            .get_by_role_and_label(Role::Tab, &entry_name(&harness, page))
            .rect()
    };

    // The 6 point top inset of each highlight fills the 36 point pitch, so
    // the rows touch and no gap remains between their rows.
    let gap = rect(PageId::Components).top() - rect(PageId::Setup).bottom();
    assert!(gap.abs() < 0.5, "packed: {gap} between entries");

    assert_unscrolled_and_uncut(&harness);
}

#[test]
fn a_shorter_window_scrolls_with_a_fade_at_the_cut_edge() {
    let harness = app(600.0, false);

    assert!(
        !in_sight_unscrolled(&harness, PageId::Diagnostics),
        "this test needs a cut list"
    );

    // The entries under the cut draw at full strength, so the cover alone
    // shows the fade.
    let bottom = cut::cut_edges(&harness)
        .into_iter()
        .fold(f32::MIN, f32::max);

    let fades = cut::fades(&harness);
    assert!(
        fades
            .iter()
            .any(|fade| (fade.bottom() - bottom).abs() < 1.5),
        "no cover meets the cut edge at {bottom}, only {fades:?}"
    );
}

#[test]
fn a_packed_group_heading_has_more_room_above_than_below() {
    let harness = app(HEIGHT, false);

    for (heading, over, under) in packed_heading_gaps(&harness) {
        assert!(
            over >= under + 2.0,
            "{heading}: {over} above against {under} below"
        );
    }
}

/// A short window packs the list, and a heading still keeps 10 points to
/// the entry above and 4 to the entry below.
#[test]
fn a_packed_group_heading_keeps_ten_above_and_four_below() {
    let harness = app(HEIGHT, false);

    for (heading, over, under) in packed_heading_gaps(&harness) {
        assert!(over >= 10.0 - 0.5, "{heading}: {over} above");
        assert!(under >= 4.0 - 0.5, "{heading}: {under} below");
    }
}

/// The packed spacing fits a window of 902 points.
#[test]
fn a_window_of_902_shows_every_entry_packed_without_scrolling() {
    let harness = app(902.0, false);

    assert_unscrolled_and_uncut(&harness);
}

/// The reading note takes a row of the sidebar, so the list packs to make
/// room for it. Every entry still shows without a scroll.
#[test]
fn a_window_of_964_shows_every_entry_while_reading() {
    let (mut harness, _runtime, _answer, _reads) = crate::system::app_at(964.0);
    crate::system::step_until(&mut harness, "the reading notice", |harness| {
        harness.query_all_by_label("Reading the system…").count() == 1
    });
    harness.run();

    assert_unscrolled_and_uncut(&harness);
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
