use egui::{Rect, Shape};
use egui_kittest::Harness;
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};
use toolbox_gui_kit::testing::painted_text;
use toolbox_gui_kit::theme::SIDEBAR_WIDTH;
use toolbox_gui_kit::tokens;

use super::cut::{Rail, collapsed_rail, cover_at};
use crate::common::{HEIGHT, SMALLEST, launch_in, window};

/// Its strength runs from 0 (gone) to 1 (whole).
struct Icon {
    entry: Rect,
    shown: Rect,
    strength: f32,
}

/// The rects the last frame painted in the sidebar, `textured` for the
/// icons or not for the fills behind the entries.
fn sidebar_rects(harness: &Harness<'_, App>, textured: bool) -> Vec<(Rect, egui::Color32)> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let Shape::Rect(rect) = &clipped.shape else {
                return None;
            };

            (rect.brush.is_some() == textured && rect.rect.right() <= SIDEBAR_WIDTH)
                .then_some((rect.rect, rect.fill))
        })
        .collect()
}

#[test]
fn the_open_entry_near_a_bottom_cut_keeps_its_whole_strength() {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        for page in [PageId::Boot, PageId::Maintenance, PageId::Diagnostics] {
            let setup = Setup {
                open: Some(page),
                ..Setup::default()
            };

            let mut harness = launch_in(window(HEIGHT).with_theme(theme), setup);
            harness.run();
            let tokens = tokens::current(&harness.ctx);
            let name = toolbox_gui::i18n::text(&harness.ctx, page.text_id());
            let text = painted_text(&harness, &name).expect("the open entry");
            let middle = text.visual_bounding_rect().center();

            let (pill, fill) = sidebar_rects(&harness, false)
                .into_iter()
                .find(|(rect, fill)| rect.contains(middle) && rect.height() < 40.0 && fill.a() > 0)
                .expect("the open entry's pill");

            let (_, tint) = sidebar_rects(&harness, true)
                .into_iter()
                .find(|(rect, _)| pill.contains(rect.center()))
                .expect("the open entry's icon");

            let label = format!("{theme:?} on {page:?}");
            assert_eq!(fill, tokens.accent_tint, "{label}: the pill fades");
            assert_eq!(tint, tokens.on_tint, "{label}: the icon fades");
            assert_eq!(
                text.fallback_color, tokens.on_tint,
                "{label}: the name fades"
            );

            let rows = (0_u8..=u8::MAX)
                .map(|step| pill.top() + f32::from(step))
                .take_while(|y| *y <= pill.bottom());

            for y in rows {
                let cover = cover_at(&harness, y);
                assert!(
                    cover < 0.01,
                    "{label}: a fade covers the pill {cover} at {y}"
                );
            }
        }
    }
}

fn rail_icons(rail: &Rail) -> Vec<Icon> {
    let harness = &rail.harness;
    let tokens = tokens::current(&harness.ctx);
    rail.entries
        .iter()
        .filter_map(|&(page, entry)| {
            let ink = if page == PageId::Home {
                tokens.on_tint
            } else {
                tokens.sidebar_text
            };

            let (icon, tint) = sidebar_rects(harness, true)
                .into_iter()
                .find(|(icon, _)| entry.contains(icon.center()))?;

            let mut shown = icon;
            shown.max.y = shown.max.y.min(rail.bottom);
            let drawn = f32::from(tint.a()) / f32::from(ink.a());
            let cover = cover_at(harness, shown.center().y);
            Some(Icon {
                entry,
                shown,
                strength: drawn * (1.0 - cover),
            })
        })
        .collect()
}

#[test]
fn the_collapsed_rail_fades_its_last_entry_above_hidden_ones() {
    for height in [SMALLEST.1, 600.0, HEIGHT] {
        let rail = collapsed_rail(height);
        let bottom = rail.bottom;
        let icons = rail_icons(&rail);

        let last = icons
            .iter()
            .filter(|icon| icon.entry.top() < bottom - 0.5)
            .max_by(|one, other| one.entry.top().total_cmp(&other.entry.top()))
            .expect("an entry above the cut edge");

        assert!(
            (0.15..=0.6).contains(&last.strength),
            "{height}: the last entry in sight at {:?} shows at {} above the cut \
             edge at {bottom}",
            last.entry,
            last.strength
        );

        for icon in icons.iter().filter(|icon| icon.entry.bottom() <= bottom) {
            assert!(
                icon.strength >= 0.15,
                "{height}: the entry at {:?} fits above {bottom} but shows at {}",
                icon.entry,
                icon.strength
            );
        }
    }
}

#[test]
fn the_collapsed_rail_leaves_no_empty_band_above_the_foot() {
    for height in [SMALLEST.1, 600.0, HEIGHT] {
        let rail = collapsed_rail(height);
        let bottom = rail.bottom;
        let reach = tokens::current(&rail.harness.ctx).sizes.nav_row_height;

        let lowest = rail_icons(&rail)
            .iter()
            .filter(|icon| icon.shown.top() < bottom && icon.strength > 0.0)
            .map(|icon| icon.shown.bottom())
            .fold(f32::MIN, f32::max);

        assert!(
            bottom - lowest <= reach,
            "{height}: the rail shows nothing from {lowest} to the cut edge at {bottom}"
        );
    }
}
