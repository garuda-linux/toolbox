//! A guide table in a frame: a rule under its header row, then zebra
//! stripes that run from edge to edge of the frame.

use egui::{CornerRadius, Frame, Grid, Margin, Rect, ScrollArea, Shape, Stroke, Ui, pos2, vec2};
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

/// The stripes are painted under the grid once its rows are measured,
/// as egui's own stop at the grid and leave the frame's margin bare.
pub(super) fn show(ui: &mut Ui, table: &Table, reader: &mut Reader<'_>) {
    let tokens = ui.tokens();
    let backdrop = ui.painter().add(Shape::Noop);
    let columns = table.head.len();
    let mut bottoms = Vec::new();
    reader.in_table = true;

    let half_gap = GAP.y / 2.0;
    let framed = Frame::new()
        .stroke(Stroke::new(1.0, tokens.outline))
        .corner_radius(CornerRadius::same(RADIUS))
        .inner_margin(Margin::symmetric(0, ROW_PAD))
        .show(ui, |ui| {
            ScrollArea::horizontal().show(ui, |ui| {
                Grid::new("table").spacing(GAP).show(ui, |ui| {
                    let rows = std::iter::once(&table.head).chain(&table.rows);
                    for (at, row) in rows.enumerate() {
                        let weight = if at == 0 {
                            Weight::SemiBold
                        } else {
                            Weight::Regular
                        };

                        let bottom = row
                            .iter()
                            .enumerate()
                            .map(|(column, cell)| {
                                let sides = (column == 0, column + 1 == columns);
                                cell_text(ui, cell, weight, sides, reader)
                            })
                            .fold(f32::MIN, f32::max);

                        bottoms.push(bottom);
                        ui.end_row();
                    }
                });
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

/// One cell's spans, with the frame's margin before the first column and
/// after the last. Returns the cell's bottom.
fn cell_text(
    ui: &mut Ui,
    cell: &[Span],
    weight: Weight,
    (first, last): (bool, bool),
    reader: &mut Reader<'_>,
) -> f32 {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if first {
            ui.add_space(SIDE);
        }

        ui.vertical(|ui| {
            ui.set_max_width(CELL_WIDTH);
            spans::show(ui, cell, weight, reader);
        });

        if last {
            ui.add_space(SIDE);
        }
    })
    .response
    .rect
    .bottom()
}
