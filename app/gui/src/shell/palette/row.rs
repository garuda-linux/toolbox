//! One palette row: its tile, its name, and its detail or its shortcut
//! as keycaps. The marked row shows as selected.

use std::sync::Arc;

use egui::{
    Atom, Atoms, Button, Color32, CornerRadius, Response, RichText, Stroke, StrokeKind, Ui, vec2,
};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::theme::SMALL_TILE_RADIUS;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{TileSize, line, paint_icon_tile, paint_letter_tile};

use super::found::{Look, Row};

const CAP_PADDING: f32 = 5.0;
const CAP_RADIUS: u8 = 4;

pub fn show(ui: &mut Ui, row: &Row, marked: bool, width: f32) -> Response {
    // The marked row's detail and keys take the colour of selected text.
    let colour = if marked {
        ui.visuals().selection.stroke.color
    } else {
        ui.visuals().weak_text_color()
    };

    let mut button =
        Button::selectable(marked, (tile(ui, row), row.name.as_str())).min_size(vec2(width, 0.0));

    // egui joins the name and an empty detail with a space, which a
    // screen reader then reads out.
    if !row.detail.is_empty() {
        button = button.right_text(RichText::new(&row.detail).color(colour));
    }

    if let Some(shortcut) = &row.shortcut {
        button = button.right_text(keycaps(ui, shortcut, colour));
    }

    let response = ui.add(button);

    // The caps are painted, so the name says the shortcut as the text
    // did before.
    if let Some(shortcut) = &row.shortcut {
        let label = format!("{} {shortcut}", row.name);
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_label(label));
    }

    response
}

fn tile<'row>(ui: &Ui, row: &Row) -> Atom<'row> {
    let edge = TileSize::Line.edge(ui);
    let size = vec2(edge, edge);
    match &row.look {
        Look::Glyph(icon, hue) => {
            let (glyph, painted) = (*icon, tokens::current(ui.ctx()).tiles.of(*hue));
            Atom::paint(size, move |painting, args| {
                paint_icon_tile(painting, args.rect.center(), TileSize::Line, glyph, painted);
            })
        }
        Look::Letter { package } => {
            let (shown, kept) = (row.name.clone(), package.clone());
            Atom::paint(size, move |painting, args| {
                paint_letter_tile(painting, args.rect, SMALL_TILE_RADIUS, &shown, &kept);
            })
        }
    }
}

/// One cap per key of `shortcut`, as the platform writes it ("Ctrl+K").
fn keycaps<'row>(ui: &Ui, shortcut: &str, ink: Color32) -> Atoms<'row> {
    let edge = Stroke::new(1.0, tokens::current(ui.ctx()).outline);
    let mut caps = Atoms::default();
    for key in shortcut.split('+').filter(|key| !key.is_empty()) {
        let galley = line(ui, key, Weight::Regular, SMALL, ink);
        let room = galley.size() + vec2(CAP_PADDING * 2.0, CAP_PADDING);
        let size = vec2(room.x.max(room.y), room.y);
        caps.push_right(Atom::paint(size, move |painting, args| {
            let painter = painting.painter();
            painter.rect_stroke(
                args.rect,
                CornerRadius::same(CAP_RADIUS),
                edge,
                StrokeKind::Inside,
            );
            let at = args.rect.center() - galley.size() / 2.0;
            painter.galley(at, Arc::clone(&galley), ink);
        }));
    }

    caps
}
