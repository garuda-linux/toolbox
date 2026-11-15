use egui::{Align, Id, Label, Layout, Sense, Ui, Vec2, vec2};

use super::icon_tile::{TileSize, page_tile, paint_icon_tile};
use super::layout::{MIN_TEXT_WIDTH, last_descent, raised_column, remember_width};
use super::list::memory::{self, RowMemory};
use super::{Tone, queued_tag, rich, status_dot};
use crate::fonts::{self, BODY, SMALL, Weight};
use crate::icons::Icon;
use crate::tokens::{self, Tile};

const WIDTH_REASON: &str = "action_row width";
const HEIGHT_REASON: &str = "action_row height";
/// A row whose words are taller than the title and about three lines of
/// text keeps its actions on the title line.
const TALL_ROWS: f32 = 3.0;

/// The tile, the words and the actions are centred on the row. A row too
/// narrow for both puts the actions on a line under the words.
#[derive(Clone, Copy, Debug)]
#[must_use = "a row shows nothing until `show` is called"]
pub struct ActionRow<'text> {
    icon: Icon,
    tone: Tone,
    title: &'text str,
    detail: &'text str,
    queued: &'text str,
    status: Option<(&'text str, Tone)>,
    row: Option<Id>,
}

impl<'text> ActionRow<'text> {
    pub const fn new(icon: Icon, tone: Tone, title: &'text str) -> Self {
        Self {
            icon,
            tone,
            title,
            detail: "",
            queued: "",
            status: None,
            row: None,
        }
    }

    /// The id of the list row this sits in, from [`RowState::row`], so its
    /// sizes are dropped with the row.
    ///
    /// [`RowState::row`]: super::RowState::row
    pub const fn row(mut self, row: Id) -> Self {
        self.row = Some(row);
        self
    }

    /// Empty shows none.
    pub const fn detail(mut self, detail: &'text str) -> Self {
        self.detail = detail;
        self
    }

    /// A [`status_dot`] with `label` beside the title, such as "Not installed".
    pub const fn status(mut self, label: &'text str, tone: Tone) -> Self {
        self.status = Some((label, tone));
        self
    }

    /// The [`queued_tag`] beside the title of a row whose change waits for
    /// Apply. Empty shows none.
    pub const fn queued(mut self, label: &'text str) -> Self {
        self.queued = label;
        self
    }

    pub fn show(self, ui: &mut Ui, actions: impl FnOnce(&mut Ui)) {
        self.draw(ui, Some(actions), None::<fn(&mut Ui)>);
    }

    /// [`show`](Self::show) with `body` drawn under the detail, in the
    /// words' column, for the links and tags a row lists under its text.
    pub fn show_with_body(
        self,
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui),
        actions: impl FnOnce(&mut Ui),
    ) {
        self.draw(ui, Some(actions), Some(body));
    }

    pub fn show_without_actions(self, ui: &mut Ui) {
        self.draw(ui, None::<fn(&mut Ui)>, None::<fn(&mut Ui)>);
    }

    fn draw(
        self,
        ui: &mut Ui,
        actions: Option<impl FnOnce(&mut Ui)>,
        body: Option<impl FnOnce(&mut Ui)>,
    ) {
        // The actions' width comes from the last frame: they are laid out
        // after the words, which must know the room left for them.
        let row = self
            .row
            .unwrap_or_else(|| ui.next_auto_id().with("action_row"));

        let kept = memory::get(ui, row);
        let known = kept.actions_width;
        let actions_width = known.unwrap_or_default();
        let sizes = tokens::current(ui.ctx()).sizes;
        let (tile_gap, actions_gap) = (sizes.picture_gap, sizes.text_to_control);
        let edge = TileSize::Row.edge(ui);
        let words_room = ui.available_width() - edge - tile_gap - actions_width - actions_gap;

        let beside = actions.is_some() && words_room >= MIN_TEXT_WIDTH;

        // The row is as tall as the tile, last frame's words or last frame's
        // actions. Tile, words and actions all centre on it.
        let (height_was, words_was) = (kept.actions_height, kept.words);
        let anchored = words_was.is_some_and(|words| words > TALL_ROWS * title_line(ui));

        // Words that reach below the tile end on a baseline, so the tile and
        // the capitals come down by up to the last line's descent and leave
        // as much room above them as under that baseline.
        let drop = words_was.map_or(0.0, |words| (words - edge).clamp(0.0, last_descent(ui)));
        let rise = capitals_top(ui) - drop;
        let raised_id = row.with("raised");
        let actions_height = height_was.filter(|_| beside).unwrap_or_default();

        let words_height = words_was.unwrap_or_default() + drop;
        let line = edge.max(actions_height).max(words_height);

        // Tile and words centre on the row beside the actions, so a single
        // line of words sits level with the tile and the actions alike.
        // Only they come down: the actions centre on the line by themselves.
        let block = edge.max(words_height);
        let lower = if beside {
            ((line - block) / 2.0).max(0.0)
        } else {
            0.0
        };

        let drawn = ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = tile_gap;
            let (slot, _) = ui.allocate_exact_size(Vec2::splat(edge), Sense::hover());
            ui.spacing_mut().item_spacing.x = actions_gap;
            if !beside {
                let width = ui.available_width();
                raised(ui, (rise, width), raised_id, |ui| {
                    self.stacked(ui, (actions, body), (row, known));
                });
                return slot;
            }

            let words = raised(ui, (rise - lower, words_room), raised_id, |ui| {
                ui.set_width(words_room);
                self.words(ui, body);
            });

            remember_width(
                ui,
                (row, |kept_row: &mut RowMemory| &mut kept_row.words),
                words_was,
                words - drop - lower,
                HEIGHT_REASON,
            );

            if let Some(actions) = actions {
                let measures = Measures {
                    row,
                    width: known,
                    height: height_was,
                };

                actions_beside(ui, line, measures, anchored, actions);
            }

            slot
        });

        self.badge(ui, drawn.inner.center() + vec2(0.0, drop + lower));
    }

    fn stacked(
        self,
        ui: &mut Ui,
        (actions, body): (Option<impl FnOnce(&mut Ui)>, Option<impl FnOnce(&mut Ui)>),
        (row, known): (Id, Option<f32>),
    ) {
        ui.vertical(|ui| {
            self.words(ui, body);
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
                remember_width(
                    ui,
                    (row, |kept_row: &mut RowMemory| &mut kept_row.actions_width),
                    known,
                    width,
                    WIDTH_REASON,
                );
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

    fn words(self, ui: &mut Ui, body: Option<impl FnOnce(&mut Ui)>) {
        let tokens = tokens::current(ui.ctx());
        ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
        let title = rich(ui, self.title, Weight::SemiBold, BODY, tokens.text);
        if self.queued.is_empty() && self.status.is_none() {
            ui.label(title);
        } else {
            ui.scope(|ui| {
                // `horizontal` reads its least height, the interact height
                // taller than the title, before its closure runs. Set inside
                // it, a queued row still grows.
                ui.spacing_mut().interact_size.y = 0.0;
                ui.horizontal(|ui| {
                    ui.label(title);
                    if let Some((label, tone)) = self.status {
                        status_dot(ui, label, tone);
                    }

                    if !self.queued.is_empty() {
                        queued_tag(ui, self.queued, Tone::Accent);
                    }
                });
            });
        }

        if !self.detail.is_empty() {
            let detail = rich(ui, self.detail, Weight::Regular, SMALL, tokens.secondary);
            ui.add(Label::new(detail).wrap());
        }

        if let Some(body) = body {
            body(ui);
        }
    }
}

#[derive(Clone, Copy)]
struct Measures {
    row: Id,
    width: Option<f32>,
    height: Option<f32>,
}

/// Keeps the actions in order at the right of a line that never wraps.
fn actions_beside(
    ui: &mut Ui,
    line: f32,
    measures: Measures,
    anchored: bool,
    actions: impl FnOnce(&mut Ui),
) {
    let (row, width_was, height_was) = (measures.row, measures.width, measures.height);
    let room = vec2(ui.available_width(), 0.0);

    // A tall row keeps the actions beside its title, not beside its middle.
    let drop = match height_was {
        Some(height) if anchored => (capital_height(ui) / 2.0 - height / 2.0).max(0.0),
        Some(height) => (line - height) / 2.0,
        None => 0.0,
    };

    let drawn = ui.allocate_ui_with_layout(room, Layout::top_down(Align::Min), |ui| {
        ui.add_space(drop);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing =
                Vec2::splat(tokens::current(ui.ctx()).sizes.control_gap);
            actions(ui);
        })
        .response
        .rect
    });

    let taken = drawn.inner;
    remember_width(
        ui,
        (row, |kept_row: &mut RowMemory| &mut kept_row.actions_width),
        width_was,
        taken.width(),
        WIDTH_REASON,
    );
    remember_width(
        ui,
        (row, |kept_row: &mut RowMemory| &mut kept_row.actions_height),
        height_was,
        taken.height(),
        HEIGHT_REASON,
    );
}

/// The line height of the title, the text of a row's first line.
fn title_line(ui: &Ui) -> f32 {
    let line = ui.painter().layout_no_wrap(
        "H".to_owned(),
        fonts::font(ui.style(), Weight::SemiBold, BODY),
        egui::Color32::PLACEHOLDER,
    );

    line.size().y
}

/// The height of a capital of the title, as the centre of a row's title.
fn capital_height(ui: &Ui) -> f32 {
    let capital = ui.painter().layout_no_wrap(
        "H".to_owned(),
        fonts::font(ui.style(), Weight::SemiBold, BODY),
        egui::Color32::PLACEHOLDER,
    );

    capital.mesh_bounds.height()
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
    let (mut child, room) = raised_column(ui, (rise, width), id);
    body(&mut child);

    let below = (child.min_rect().bottom() - room.top()).max(0.0);
    ui.allocate_exact_size(vec2(width, below), Sense::hover());
    below
}
