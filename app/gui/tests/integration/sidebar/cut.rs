//! What the sidebar's list shows at a cut edge: a soft fade into the
//! sidebar, and no group heading cut in half.

use egui::Rect;
use egui_kittest::Harness;
use toolbox_gui::page::{Group, PageId};
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::testing::{lines_in, painted_text};
use toolbox_gui_kit::theme::SIDEBAR_WIDTH;
use toolbox_gui_kit::tokens;

use crate::common::{HEIGHT, SMALLEST, launch_in, window};

const HEADINGS: [&str; 3] = ["INSTALL", "SYSTEM", "CARE"];

/// The collapsed rail at some height: its entries with their rects, the
/// heights of the dividers drawn above its bottom cut edge, and that edge.
pub(super) struct Rail {
    pub(super) harness: Harness<'static, App>,
    pub(super) entries: Vec<(PageId, Rect)>,
    pub(super) dividers: Vec<f32>,
    pub(super) bottom: f32,
}

pub(super) fn app_on(height: f32, page: PageId) -> Harness<'static, App> {
    let setup = Setup {
        open: Some(page),
        ..Setup::default()
    };

    let mut harness = launch_in(window(height), setup);
    harness.run();
    harness
}

fn cut_edges(harness: &Harness<'_, App>) -> Vec<f32> {
    let edge = tokens::current(&harness.ctx).sidebar_edge;
    lines_in(harness, edge)
        .iter()
        .filter(|[start, end]| {
            (start.y - end.y).abs() < 0.5
                && end.x <= SIDEBAR_WIDTH + 1.0
                && end.x - start.x > SIDEBAR_WIDTH / 2.0
        })
        .map(|[start, _]| start.y)
        .collect()
}

/// The gradients the last frame laid over the sidebar from its own fill
/// to nothing.
fn fades(harness: &Harness<'_, App>) -> Vec<Rect> {
    let sidebar = tokens::current(&harness.ctx).sidebar;
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let egui::Shape::Mesh(mesh) = &clipped.shape else {
                return None;
            };

            let colours: Vec<_> = mesh.vertices.iter().map(|vertex| vertex.color).collect();
            let rect = mesh.calc_bounds();
            (rect.right() <= SIDEBAR_WIDTH + 1.0
                && colours.contains(&sidebar)
                && colours.contains(&egui::Color32::TRANSPARENT))
            .then_some(rect)
        })
        .collect()
}

#[test]
fn a_cut_edge_fades_softly_into_the_sidebar() {
    for page in [PageId::Home, PageId::Maintenance] {
        let harness = app_on(HEIGHT, page);
        let edges = cut_edges(&harness);
        let fades = fades(&harness);

        assert!(!edges.is_empty(), "{page:?}: this test needs a cut list");
        for edge in edges {
            assert!(
                fades
                    .iter()
                    .any(|fade| (fade.top() - edge).abs() < 1.5
                        || (fade.bottom() - edge).abs() < 1.5),
                "{page:?}: no fade meets the cut edge at {edge}, only {fades:?}"
            );
        }
    }
}

#[test]
fn an_uncut_list_lays_no_fade() {
    let harness = app_on(796.0, PageId::Home);

    assert!(fades(&harness).is_empty(), "{:?}", fades(&harness));
}

#[test]
fn no_group_heading_shows_cut_by_an_edge() {
    for (height, page) in [
        (HEIGHT, PageId::Home),
        (HEIGHT, PageId::Maintenance),
        (600.0, PageId::Home),
        (600.0, PageId::Diagnostics),
    ] {
        let harness = app_on(height, page);
        for edge in cut_edges(&harness) {
            for heading in HEADINGS {
                let Some(text) = painted_text(&harness, heading) else {
                    continue;
                };

                let rect = text.galley.rect.translate(text.pos.to_vec2());
                assert!(
                    rect.bottom() <= edge + 0.5 || rect.top() >= edge - 0.5,
                    "{height} on {page:?}: {heading} at {rect:?} is cut at {edge}"
                );
            }
        }
    }
}

/// How far the fades cover the sidebar at the height `y`, from 0 (clear)
/// to 1 (the sidebar's fill alone).
pub(super) fn cover_at(harness: &Harness<'_, App>, y: f32) -> f32 {
    let sidebar = tokens::current(&harness.ctx).sidebar;
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let egui::Shape::Mesh(mesh) = &clipped.shape else {
                return None;
            };

            (mesh.calc_bounds().right() <= SIDEBAR_WIDTH + 1.0
                && mesh.vertices.iter().any(|vertex| vertex.color == sidebar))
            .then_some(mesh)
        })
        .filter_map(|mesh| {
            let left = mesh.calc_bounds().left();
            let mut column: Vec<_> = mesh
                .vertices
                .iter()
                .filter(|vertex| (vertex.pos.x - left).abs() < 0.5)
                .map(|vertex| (vertex.pos.y, f32::from(vertex.color.a()) / 255.0))
                .collect();

            column.sort_by(|one, other| one.0.total_cmp(&other.0));
            column.windows(2).find_map(|pair| {
                let &[(top, over), (bottom, under)] = pair else {
                    return None;
                };

                (top <= y && y <= bottom && bottom > top)
                    .then(|| egui::lerp(over..=under, (y - top) / (bottom - top)))
            })
        })
        .fold(0.0, f32::max)
}

/// The contrast the last frame left between the sidebar and the name of
/// its last entry above the bottom cut edge, faded and covered as drawn.
fn last_entry_contrast(harness: &Harness<'_, App>) -> f32 {
    let tokens = tokens::current(&harness.ctx);
    let edge = cut_edges(harness).into_iter().fold(f32::MIN, f32::max);

    let name = harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let egui::Shape::Text(text) = &clipped.shape else {
                return None;
            };

            (text.pos.x < SIDEBAR_WIDTH).then_some(text)
        })
        .map(|text| (text.visual_bounding_rect().center().y, text.fallback_color))
        .filter(|(middle, _)| *middle < edge)
        .max_by(|one, other| one.0.total_cmp(&other.0))
        .expect("an entry above the cut edge");

    let (middle, ink) = name;
    let shown = tokens.sidebar.blend(ink);
    let cover = tokens.sidebar.gamma_multiply(cover_at(harness, middle));
    tokens::contrast(shown.blend(cover), tokens.sidebar)
}

/// The entry above a heading hidden at the bottom edge fades to about
/// 2:1 against the sidebar, in dark as in light.
#[test]
fn the_last_entry_above_a_bottom_cut_fades_alike_in_both_themes() {
    let contrast = |theme| {
        let harness = launch_in(window(HEIGHT).with_theme(theme), Setup::default());
        last_entry_contrast(&harness)
    };

    let (dark, light) = (contrast(egui::Theme::Dark), contrast(egui::Theme::Light));
    for (theme, shown) in [("dark", dark), ("light", light)] {
        assert!(
            shown <= 2.2,
            "{theme}: the last entry still shows at {shown}:1"
        );
    }

    assert!(
        (dark - light).abs() <= 0.4,
        "dark {dark}:1 against light {light}:1"
    );
}

/// The lines across the collapsed rail, faded or not, so a divider drawn
/// at a cut edge counts too.
fn rail_lines(harness: &Harness<'_, App>) -> Vec<[egui::Pos2; 2]> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let egui::Shape::LineSegment {
                points: [left, right],
                stroke,
            } = clipped.shape
            else {
                return None;
            };

            (stroke.color.a() > 0 && (left.y - right.y).abs() < 0.5 && right.x < 80.0)
                .then_some([left, right])
        })
        .collect()
}

pub(super) fn collapsed_rail(height: f32) -> Rail {
    use egui::accesskit::Role;
    use egui_kittest::kittest::Queryable as _;

    let mut harness = app_on(height, PageId::Home);
    harness.get_by_label("Collapse sidebar").click();
    harness.run();
    harness.run();

    let entries: Vec<(PageId, Rect)> = PageId::ALL
        .into_iter()
        .filter(|page| *page != PageId::BootRepair)
        .map(|page| {
            let name = toolbox_gui::i18n::text(&harness.ctx, page.text_id());
            (page, harness.get_by_role_and_label(Role::Tab, &name).rect())
        })
        .collect();

    let half = entries[0].1.width() / 2.0;
    let (dividers, edges): (Vec<_>, Vec<_>) = rail_lines(&harness)
        .into_iter()
        .partition(|[left, right]| right.x - left.x < half);

    let bottom = edges
        .iter()
        .map(|[left, _]| left.y)
        .filter(|y| *y > height / 2.0)
        .fold(f32::INFINITY, f32::min);

    let dividers = dividers
        .into_iter()
        .map(|[left, _]| left.y)
        .filter(|y| *y < bottom)
        .collect();

    Rail {
        harness,
        entries,
        dividers,
        bottom,
    }
}

/// A collapsed rail parts two groups with a line exactly while any part
/// of the second group's first entry shows above the cut edge. That entry
/// then draws cut and faded, so the rail shows that more follows, and no
/// line hangs at the edge with nothing below.
#[test]
fn a_collapsed_divider_shows_while_its_next_entry_reaches_into_sight() {
    for height in [SMALLEST.1, 600.0, HEIGHT] {
        let rail = collapsed_rail(height);
        let bottom = rail.bottom;
        let listed: Vec<_> = rail
            .entries
            .iter()
            .filter(|(page, _)| page.group() != Group::App)
            .collect();

        for &line in &rail.dividers {
            let next = listed
                .iter()
                .map(|(_, rect)| rect)
                .filter(|entry| entry.top() >= line)
                .min_by(|one, other| one.top().total_cmp(&other.top()));

            if let Some(next) = next {
                assert!(
                    next.top() < bottom - 0.5,
                    "{height}: the line at {line} parts nothing, its next entry \
                     starts at {} past the cut edge at {bottom}",
                    next.top()
                );
            }
        }

        for pair in listed.windows(2) {
            let &[(before, above), (page, first)] = pair else {
                continue;
            };

            let parted = before.group() != page.group() && page.group().heading_id().is_some();
            if parted && first.top() < bottom - 0.5 {
                assert!(
                    rail.dividers
                        .iter()
                        .any(|line| (above.bottom()..first.top()).contains(line)),
                    "{height}: no line above {page:?} at {first:?}, which reaches \
                     above the cut edge at {bottom}"
                );
            }
        }
    }
}
