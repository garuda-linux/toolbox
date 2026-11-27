use egui::accesskit::Role;
use egui::{Event, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{SIDE_INSET, highlight_rect};

use super::entry_name;
use super::head::assert_edges_meet;
use super::strength::sidebar_rects;
use crate::common::{HEIGHT, launch_in, window};

/// The middles of an entry's icon, of its label's capitals (the top of the
/// ink down to the baseline, so the descenders do not pull it) and of its
/// highlight. Only the label is absent on the collapsed rail.
struct Middles {
    icon: f32,
    text: Option<f32>,
    highlight: f32,
}

#[test]
fn the_open_entry_highlight_sits_inside_its_row() {
    let harness = open_harness(Some(PageId::Home));

    let tokens = tokens::current(&harness.ctx);
    let name = entry_name(&harness, PageId::Home);
    let row = harness.get_by_role_and_label(Role::Tab, &name).rect();
    let (pill, _) = sidebar_rects(&harness, false)
        .into_iter()
        .find(|(rect, fill)| rect.contains(row.center()) && *fill == tokens.accent_tint)
        .expect("the open entry's highlight");

    assert_edges_meet([
        ("left", pill.left(), row.left() + SIDE_INSET),
        ("right", pill.right(), row.right() - SIDE_INSET),
        ("top", pill.top(), row.top() + SIDE_INSET),
        ("bottom", pill.bottom(), row.bottom()),
    ]);
    assert!((pill.height() - 30.0).abs() < 0.5, "{pill:?}");
}

fn middles(harness: &Harness<'_, App>, page: PageId) -> Middles {
    let name = entry_name(harness, page);
    let row = harness.get_by_role_and_label(Role::Tab, &name).rect();
    let highlight = highlight_rect(row).center().y;
    let icon = sidebar_rects(harness, true)
        .into_iter()
        .find(|(rect, _)| row.contains(rect.center()))
        .map(|(rect, _)| rect.center().y)
        .expect("the entry's icon");

    let text = harness.output().shapes.iter().find_map(|clipped| {
        let Shape::Text(text) = &clipped.shape else {
            return None;
        };

        if text.galley.text() != name || !row.contains(text.pos) {
            return None;
        }

        let placed = text.galley.rows.first()?;
        let glyph = placed.row.glyphs.first()?;
        let baseline = text.pos.y + placed.pos.y + glyph.pos.y;
        let cap_top = text.pos.y + text.galley.mesh_bounds.top();
        Some(f32::midpoint(cap_top, baseline))
    });

    Middles {
        icon,
        text,
        highlight,
    }
}

fn open_harness(open: Option<PageId>) -> Harness<'static, App> {
    let mut harness = launch_in(
        window(HEIGHT),
        Setup {
            open,
            ..Setup::default()
        },
    );

    harness.run();
    harness
}

fn point_at(harness: &mut Harness<'static, App>, page: PageId) {
    let name = entry_name(harness, page);
    let row = harness.get_by_role_and_label(Role::Tab, &name).rect();
    harness.event(Event::PointerMoved(row.center()));
    harness.run();
}

/// Icon, label and highlight share one middle for the selected, the
/// hovered and the rest entry, expanded and collapsed.
#[test]
fn the_icon_and_label_sit_on_the_highlight_middle() {
    for expanded in [true, false] {
        let mut harness = open_harness(Some(PageId::Gaming));
        if !expanded {
            harness.get_by_label("Collapse sidebar").click();
            harness.run();
            harness.run();
        }

        point_at(&mut harness, PageId::Components);
        let hovered = middles(&harness, PageId::Components);
        let rest = middles(&harness, PageId::Setup);
        let selected = middles(&harness, PageId::Gaming);
        for (label, entry) in [
            ("selected", &selected),
            ("hovered", &hovered),
            ("rest", &rest),
        ] {
            let at = format!("{label}, expanded {expanded}");
            assert!(
                (entry.icon - entry.highlight).abs() <= 0.5,
                "{at}: icon {} in highlight {}",
                entry.icon,
                entry.highlight
            );

            match entry.text {
                Some(text) => assert!(
                    (text - entry.highlight).abs() <= 0.5,
                    "{at}: text {text} in highlight {}",
                    entry.highlight
                ),
                None => assert!(!expanded, "{at}: the label is missing"),
            }
        }
    }
}
