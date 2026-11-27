use egui::{Align, Id, Label, Layout, Sense, Ui, Vec2, pos2, vec2};

use super::icon_tile::{ICON_EDGE, paint_icon};
use super::layout::{self, raised_column, remember_width};
use super::list::memory::{self, RowMemory};
use super::{Tone, queued_tag, rich, status_dot};
use crate::fonts::{self, BODY, SMALL, Weight};
use crate::icons::Icon;
use crate::tokens;

const WIDTH_REASON: &str = "action_row width";
const HEIGHT_REASON: &str = "action_row height";
/// A row whose words are taller than its title and this many detail lines,
/// with the gap under each, keeps its actions on the title line.
const TALL_DETAIL: f32 = 2.5;

/// The tile, the words and the actions are centred on the row. A row too
/// narrow for both puts the actions on a line under the words.
#[derive(Clone, Copy, Debug)]
#[must_use = "a row shows nothing until `show` is called"]
pub struct ActionRow<'text> {
    icon: Icon,
    title: &'text str,
    detail: &'text str,
    queued: &'text str,
    status: Option<(&'text str, Tone)>,
    /// Beside `status`, when a row has two states such as Running and Default.
    second_status: Option<(&'text str, Tone)>,
    row: Option<Id>,
}

impl<'text> ActionRow<'text> {
    pub const fn new(icon: Icon, title: &'text str) -> Self {
        Self {
            icon,
            title,
            detail: "",
            queued: "",
            status: None,
            second_status: None,
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

    /// A second [`status_dot`] after the one of [`Self::status`].
    pub const fn second_status(mut self, label: &'text str, tone: Tone) -> Self {
        self.second_status = Some((label, tone));
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
        let edge = ICON_EDGE;
        let full = ui.available_width() - edge - tile_gap;
        let fits = layout::beside(full, actions_width, actions_gap);
        let words_room = fits.filter(|_| actions.is_some());

        // The row is as tall as its tallest part. The tile and the words centre
        // on it, as in KDE form delegates. Stacked under the words the actions
        // stay out of that centring.
        let (height_was, words_was) = (kept.actions_height, kept.words);
        let anchored = is_anchored(ui, words_was);
        set_stacked(ui, row, words_room.is_none());
        let raised_id = row.with("raised");
        let words_height = words_was.unwrap_or_default();
        let actions_height = words_room.and(height_was).unwrap_or_default();
        let line = edge.max(actions_height).max(words_height);
        let block = if words_room.is_some() {
            line
        } else {
            edge.max(words_height)
        };

        let words_drop = ((block - words_height) / 2.0).max(0.0);
        let top = ui.cursor().top();

        let drawn = ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = tile_gap;
            let (slot, _) = ui.allocate_exact_size(Vec2::splat(edge), Sense::hover());
            ui.spacing_mut().item_spacing.x = actions_gap;
            let Some(words_room) = words_room else {
                let width = ui.available_width();
                raised(ui, (-words_drop, width), raised_id, |ui| {
                    self.stacked(ui, (actions, body), (row, known, words_was));
                });
                return slot;
            };

            let words = raised(ui, (-words_drop, words_room), raised_id, |ui| {
                ui.set_width(words_room);
                self.words(ui, body);
            });

            remember_width(
                ui,
                (row, |kept_row: &mut RowMemory| &mut kept_row.words),
                words_was,
                words - words_drop,
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

        let centre = pos2(drawn.inner.center().x, top + block / 2.0);
        self.badge(ui, centre);
    }

    fn stacked(
        self,
        ui: &mut Ui,
        (actions, body): (Option<impl FnOnce(&mut Ui)>, Option<impl FnOnce(&mut Ui)>),
        (row, known, words_was): (Id, Option<f32>, Option<f32>),
    ) {
        ui.vertical(|ui| {
            self.words(ui, body);
            remember_width(
                ui,
                (row, |kept_row: &mut RowMemory| &mut kept_row.words),
                words_was,
                ui.min_rect().height(),
                HEIGHT_REASON,
            );

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

                // A one line row keeps the width measured beside the words, or
                // a narrower stacked measure sends it back beside them each frame.
                let width = if one_line { rect.width() } else { room };
                if !one_line || known.is_none() {
                    remember_width(
                        ui,
                        (row, |kept_row: &mut RowMemory| &mut kept_row.actions_width),
                        known,
                        width,
                        WIDTH_REASON,
                    );
                }
            }
        });
    }

    /// A status row passes its status icon itself.
    fn badge(self, ui: &Ui, centre: egui::Pos2) {
        paint_icon(ui, centre, self.icon.for_row());
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

                    if let Some((label, tone)) = self.second_status {
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
    let room = ui.available_width();

    // A tall row centres the actions on its title's capitals, which sit
    // above the middle of the row, so the actions may rise above its top.
    // A negative rise drops them below the cursor, as `raised` does.
    let rise = match height_was {
        Some(height) if anchored => height / 2.0 - capital_middle(ui),
        Some(height) => (height - line) / 2.0,
        None => 0.0,
    };

    // Only the width and the height below the cursor are allocated, so the
    // controls after the words keep the rest of the row.
    let (mut column, room_rect) = raised_column(ui, (rise, room), row.with("actions"));

    // Centred on the tallest control of the last frame, so a note before a
    // button sits on the button's middle.
    let line_height = height_was.unwrap_or(0.0);
    let taken = column
        .allocate_ui_with_layout(
            vec2(column.available_width(), line_height),
            Layout::left_to_right(Align::Center),
            |controls| {
                controls.spacing_mut().item_spacing =
                    Vec2::splat(tokens::current(controls.ctx()).sizes.control_gap);
                actions(controls);
            },
        )
        .response
        .rect;

    let below = (column.min_rect().bottom() - room_rect.top()).max(0.0);
    ui.allocate_exact_size(vec2(taken.width(), below), Sense::hover());

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

/// Whether the actions of the row `row` sit under its words in this frame,
/// rather than beside them. Set by the row before its actions are drawn.
pub fn actions_stacked(ui: &Ui, row: Id) -> bool {
    ui.data(|data| data.get_temp::<bool>(stacked_id(row)))
        .unwrap_or(false)
}

fn is_anchored(ui: &Ui, words_was: Option<f32>) -> bool {
    let gap = tokens::current(ui.ctx()).sizes.tight_gap;
    let font = fonts::font(ui.style(), Weight::Regular, SMALL);
    let detail = ui.ctx().fonts_mut(|fonts| fonts.row_height(&font));
    let limit = TALL_DETAIL.mul_add(detail, 2.0_f32.mul_add(gap, title_line(ui)));
    words_was.is_some_and(|words| words > limit)
}

/// Read by the actions themselves, which may leave out a slot that only
/// keeps room beside the words.
fn set_stacked(ui: &Ui, row: Id, stacked: bool) {
    ui.data_mut(|data| data.insert_temp(stacked_id(row), stacked));
}

fn stacked_id(row: Id) -> Id {
    row.with("actions stacked")
}

fn title_capital(ui: &Ui) -> std::sync::Arc<egui::Galley> {
    ui.painter().layout_no_wrap(
        "H".to_owned(),
        fonts::font(ui.style(), Weight::SemiBold, BODY),
        egui::Color32::PLACEHOLDER,
    )
}

/// The line height of the title, the text of a row's first line.
fn title_line(ui: &Ui) -> f32 {
    title_capital(ui).size().y
}

/// Where the middle of a title's capitals lies below the top of the title's
/// line. The line box has room above the capitals, so half the capital's
/// height is not the middle.
fn capital_middle(ui: &Ui) -> f32 {
    let capital = title_capital(ui);
    f32::midpoint(capital.mesh_bounds.top(), capital.mesh_bounds.bottom())
}

/// `body` drawn `rise` points above the cursor, or below it for a negative
/// `rise`, in a column `width` wide, taking room only for what reaches
/// below the cursor. Returns that height, `rise` included.
fn raised(ui: &mut Ui, (rise, width): (f32, f32), id: Id, body: impl FnOnce(&mut Ui)) -> f32 {
    let (mut child, room) = raised_column(ui, (rise, width), id);
    body(&mut child);

    let below = (child.min_rect().bottom() - room.top()).max(0.0);
    ui.allocate_exact_size(vec2(width, below), Sense::hover());
    below
}
