//! A guide table in a frame: a rule under its header row, then zebra
//! stripes that run from edge to edge of the frame.

use egui::{
    Align, CornerRadius, Frame, Layout, Margin, Rect, ScrollArea, Shape, Stroke, Ui, UiBuilder,
    pos2, vec2,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::Weight;
use toolbox_gui_kit::theme::RADIUS;
use toolbox_guide::{Span, Table};

use super::{Reader, spans};

/// The widest a table cell runs before its text wraps.
const CELL_WIDTH: f32 = 340.0;
/// The room between the frame and the text of the outer columns.
const SIDE: f32 = 12.0;
const GAP: egui::Vec2 = vec2(20.0, 10.0);
/// Above the first row and below the last, half of [`GAP`] as the rows
/// have between them.
const ROW_PAD: i8 = 5;
/// The frame's outline, which takes its width from the cells' room.
const OUTLINE: f32 = 1.0;

/// The stripes are painted after the rows are measured, so they run into the
/// frame's margin.
pub(super) fn show(ui: &mut Ui, table: &Table, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let backdrop = ui.painter().add(Shape::Noop);
    let columns = table.head.len();

    // The cells share the pane's width, so the table wraps instead of
    // running sideways, up to a cell's `CELL_WIDTH`.
    let counted = f32::from(u16::try_from(columns.max(1)).unwrap_or(u16::MAX));
    let beside = 2.0_f32.mul_add(SIDE + OUTLINE, GAP.x * (counted - 1.0));

    // Whole points, so the columns never overflow the frame by a fraction
    // and make room for a scroll bar.
    let room = ((ui.available_width() - beside) / counted)
        .floor()
        .clamp(0.0, CELL_WIDTH);

    let mut bottoms = Vec::new();
    reader.in_table = true;

    let half_gap = GAP.y / 2.0;
    let framed = Frame::new()
        .stroke(Stroke::new(OUTLINE, tokens.outline))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::symmetric(0, ROW_PAD))
        .show(ui, |ui| {
            // Laid out by hand: egui's Grid centres a cell in a row height
            // from the last frame, so wrapped cells start at different tops.
            ScrollArea::horizontal().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let start = ui.cursor().min;
                let mut top = start.y;
                let rows = std::iter::once(&table.head).chain(&table.rows);
                for (at, row) in rows.enumerate() {
                    let weight = if at == 0 {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    };

                    if at > 0 {
                        top += GAP.y;
                    }

                    let mut left = start.x + SIDE;
                    let mut bottom = top;
                    for cell in row {
                        let slot = Rect::from_min_size(pos2(left, top), vec2(room, 0.0));
                        bottom = bottom.max(cell_text(ui, cell, weight, slot, reader));
                        left += room + GAP.x;
                    }

                    let right = left - GAP.x + SIDE;
                    ui.advance_cursor_after_rect(Rect::from_min_max(
                        pos2(start.x, top),
                        pos2(right, bottom),
                    ));
                    bottoms.push(bottom);
                    top = bottom;
                }
            });
        });

    reader.in_table = false;
    let frame = framed.response.rect;
    let edges: Vec<f32> = std::iter::once(frame.top())
        .chain(
            bottoms
                .iter()
                .take(bottoms.len().saturating_sub(1))
                .map(|bottom| bottom + half_gap),
        )
        .chain(std::iter::once(frame.bottom()))
        .collect();

    let shapes = backdrop_shapes(ui, frame, &edges);
    ui.painter().set(backdrop, Shape::Vec(shapes));
}

/// A stripe on every second body row and a rule under the header row,
/// from the band edges `edges`, the frame's top first and its bottom last.
fn backdrop_shapes(ui: &Ui, frame: Rect, edges: &[f32]) -> Vec<Shape> {
    let tokens = ui.tokens();
    let stripe = ui.visuals().faint_bg_color;
    let last = edges.len().saturating_sub(2);
    let mut shapes = Vec::new();

    for (row, band) in edges.windows(2).enumerate() {
        let [top, bottom] = band else {
            continue;
        };

        if row % 2 == 1 {
            let corners = if row == last {
                CornerRadius {
                    sw: RADIUS,
                    se: RADIUS,
                    ..CornerRadius::ZERO
                }
            } else {
                CornerRadius::ZERO
            };

            let rect = Rect::from_x_y_ranges(frame.x_range(), *top..=*bottom);
            shapes.push(Shape::rect_filled(rect, corners, stripe));
        }
    }

    if let Some(rule) = edges.get(1) {
        let stroke = Stroke::new(1.0, tokens.outline);
        shapes.push(Shape::line_segment(
            [pos2(frame.left(), *rule), pos2(frame.right(), *rule)],
            stroke,
        ));
    }

    shapes
}

/// One cell's spans from the top left of `at`, wrapped at its width.
/// Returns the cell's bottom.
fn cell_text(ui: &mut Ui, cell: &[Span], weight: Weight, at: Rect, reader: &mut Reader<'_>) -> f32 {
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(at)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.set_max_width(at.width());
            spans::show(ui, cell, weight, reader);
        },
    )
    .response
    .rect
    .bottom()
}
