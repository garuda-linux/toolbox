use egui::{Align, Id, Label, Layout, Rect, Sense, Ui, UiBuilder, Vec2, vec2};

use super::icon_tile::{TileSize, page_tile, paint_icon_tile};
use super::layout::{MIN_TEXT_WIDTH, last_descent, remember_width};
use super::{Tone, rich};
use crate::fonts::{self, BODY, SMALL, Weight};
use crate::icons::Icon;
use crate::tokens::{self, Tile};

const ACTIONS_GAP: f32 = 14.0;
const WIDTH_REASON: &str = "action_row width";
const HEIGHT_REASON: &str = "action_row height";

/// The tile and the title's capitals start together near the row's top, and the actions are centred on the whole row.
/// A row too narrow for both puts the actions on a line under the words.
#[derive(Clone, Copy, Debug)]
#[must_use = "a row shows nothing until `show` is called"]
pub struct ActionRow<'text> {
    icon: Icon,
    tone: Tone,
    title: &'text str,
    detail: &'text str,
}

impl<'text> ActionRow<'text> {
    pub const fn new(icon: Icon, tone: Tone, title: &'text str) -> Self {
        Self {
            icon,
            tone,
            title,
            detail: "",
        }
    }

    /// Empty shows none.
    pub const fn detail(mut self, detail: &'text str) -> Self {
        self.detail = detail;
        self
    }

    pub fn show(self, ui: &mut Ui, actions: impl FnOnce(&mut Ui)) {
        self.draw(ui, Some(actions));
    }

    pub fn show_without_actions(self, ui: &mut Ui) {
        self.draw(ui, None::<fn(&mut Ui)>);
    }

    fn draw(self, ui: &mut Ui, actions: Option<impl FnOnce(&mut Ui)>) {
        // The actions' width comes from the last frame: they are laid out
        // after the words, which must know the room left for them.
        let width_id = ui.next_auto_id().with("action_row_actions");
        let known = ui.data(|data| data.get_temp::<f32>(width_id));
        let actions_width = known.unwrap_or_default();
        let tile_gap = tokens::current(ui.ctx()).sizes.picture_gap;
        let words_room =
            ui.available_width() - TileSize::Row.edge(ui) - tile_gap - actions_width - ACTIONS_GAP;

        let beside = actions.is_some() && words_room >= MIN_TEXT_WIDTH;

        // The actions centre on a row as tall as the tile, last frame's
        // words or last frame's actions. Tile and words keep to the top,
        // so the title stays level with the tile however many lines follow.
        let height_id = width_id.with("height");
        let height_was = ui.data(|data| data.get_temp::<f32>(height_id));
        let words_id = width_id.with("words");
        let words_was = ui.data(|data| data.get_temp::<f32>(words_id));
        let edge = TileSize::Row.edge(ui);

        // Words that reach below the tile end on a baseline, so the tile and
        // the capitals come down by up to the last line's descent and leave
        // as much room above them as under that baseline.
        let drop = words_was.map_or(0.0, |words| (words - edge).clamp(0.0, last_descent(ui)));
        let rise = capitals_top(ui) - drop;
        let raised_id = width_id.with("raised");
        let actions_height = if beside {
            height_was.unwrap_or_default()
        } else {
            0.0
        };

        let words_height = words_was.unwrap_or_default() + drop;
        let line = edge.max(actions_height).max(words_height);

        let drawn = ui.horizontal_top(|ui| {
            // The words sit a picture gap from the tile, as in every row
            // with a picture, and the actions further off.
            ui.spacing_mut().item_spacing.x = tile_gap;
            let (slot, _) = ui.allocate_exact_size(Vec2::splat(edge), Sense::hover());
            ui.spacing_mut().item_spacing.x = ACTIONS_GAP;
            if !beside {
                let width = ui.available_width();
                raised(ui, (rise, width), raised_id, |ui| {
                    self.stacked(ui, actions, (width_id, known));
                });
                return slot;
            }

            let words_width = words_room.max(0.0);
            let words = raised(ui, (rise, words_width), raised_id, |ui| {
                ui.set_width(words_width);
                self.words(ui);
            });

            remember_width(ui, words_id, words_was, words - drop, HEIGHT_REASON);

            if let Some(actions) = actions {
                let measures = Measures {
                    width: (width_id, known),
                    height: (height_id, height_was),
                };

                actions_beside(ui, line, measures, actions);
            }

            slot
        });

        self.badge(ui, drawn.inner.center() + vec2(0.0, drop));
    }

    fn stacked(
        self,
        ui: &mut Ui,
        actions: Option<impl FnOnce(&mut Ui)>,
        (width_id, known): (Id, Option<f32>),
    ) {
        ui.vertical(|ui| {
            self.words(ui);
            if let Some(actions) = actions {
                ui.add_space(tokens::current(ui.ctx()).sizes.caption_gap);
                let room = ui.available_width();
                let drawn = ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing =
                        Vec2::splat(tokens::current(ui.ctx()).sizes.control_gap);
                    actions(ui);
                });

                // A line that wrapped never fits beside the words, so it
                // counts as the full width and the row stays like this
                // rather than flip each frame.
                let rect = drawn.response.rect;
                let one_line = rect.height() <= ui.spacing().interact_size.y * 1.5;
                let width = if one_line { rect.width() } else { room };
                remember_width(ui, width_id, known, width, WIDTH_REASON);
            }
        });
    }

    /// The icon's tile: the page's, or a status wash for a row that warns,
    /// fails or reports success, as that colour carries its meaning.
    fn badge(self, ui: &Ui, centre: egui::Pos2) {
        let tokens = tokens::current(ui.ctx());
        let tile = match self.tone {
            Tone::Accent | Tone::Neutral => page_tile(ui.ctx()),
            Tone::Warning | Tone::Success | Tone::Error => {
                let (fill, ink, _) = self.tone.banner(&tokens);
                Tile { fill, ink }
            }
        };

        paint_icon_tile(ui, centre, TileSize::Row, self.icon, tile);
    }

    fn words(self, ui: &mut Ui) {
        let tokens = tokens::current(ui.ctx());
        ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
        ui.label(rich(ui, self.title, Weight::SemiBold, BODY, tokens.text));
        if !self.detail.is_empty() {
            let detail = rich(ui, self.detail, Weight::Regular, SMALL, tokens.secondary);
            ui.add(Label::new(detail).wrap());
        }
    }
}

#[derive(Clone, Copy)]
struct Measures {
    width: (Id, Option<f32>),
    height: (Id, Option<f32>),
}

/// A line that never wraps measures the actions' own width. The room left
/// is that width, so they end at the right in their order, the same as on
/// a line under the words.
fn actions_beside(ui: &mut Ui, line: f32, measures: Measures, actions: impl FnOnce(&mut Ui)) {
    let (width_id, width_was) = measures.width;
    let (height_id, height_was) = measures.height;
    let room = vec2(ui.available_width(), 0.0);
    let drop = height_was.map_or(0.0, |height| (line - height) / 2.0);
    let drawn = ui.allocate_ui_with_layout(room, Layout::top_down(Align::Min), |ui| {
        ui.add_space(drop.max(0.0));
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing =
                Vec2::splat(tokens::current(ui.ctx()).sizes.control_gap);
            actions(ui);
        })
        .response
        .rect
    });

    let taken = drawn.inner;
    remember_width(ui, width_id, width_was, taken.width(), WIDTH_REASON);
    remember_width(ui, height_id, height_was, taken.height(), HEIGHT_REASON);
}

/// How far below the top of the title's label its capitals start. The
/// tile starts there, so it meets the letters rather than the line box.
pub(super) fn capitals_top(ui: &Ui) -> f32 {
    let capital = ui.painter().layout_no_wrap(
        "H".to_owned(),
        fonts::font(ui.style(), Weight::SemiBold, BODY),
        egui::Color32::PLACEHOLDER,
    );

    capital.mesh_bounds.top().max(0.0)
}

/// `body` drawn `rise` points above the cursor in a column `width` wide,
/// taking room only for what reaches below the cursor. Returns that
/// height.
///
/// The title's line box starts above its capitals. Raising the words by
/// that much puts the capitals, and so the tile, at the row's top, which
/// leaves the tile as much room to the card's edge above as below.
fn raised(ui: &mut Ui, (rise, width): (f32, f32), id: Id, body: impl FnOnce(&mut Ui)) -> f32 {
    let room = ui.available_rect_before_wrap();
    let column = Rect::from_min_size(
        room.min - vec2(0.0, rise),
        vec2(width, room.height() + rise),
    );

    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(id)
            .max_rect(column)
            .layout(Layout::top_down(Align::Min)),
    );

    body(&mut child);

    let below = (child.min_rect().bottom() - room.top()).max(0.0);
    ui.allocate_exact_size(vec2(width, below), Sense::hover());
    below
}
