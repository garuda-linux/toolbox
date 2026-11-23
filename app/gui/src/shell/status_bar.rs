//! The bar at the bottom of the window: a dry run, the parts the last read
//! of the system could not reach, and a read in progress.
//!
//! The bar only comes for what holds a while (a dry run, failed parts), so
//! the window's layout does not jump on every read. A read with nothing
//! else to say shows in a small note over the page's corner instead.

use egui::{Align2, Area, Context, CornerRadius, Frame, Id, Order, Panel, Rect, Ui, pos2};
use toolbox_gui_kit::frost::Glass;
use toolbox_gui_kit::theme::RADIUS;
use toolbox_gui_kit::widgets::{TileSize, Tone, icon_button, status_tile};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::protocol::Mode;
use toolbox_system::Probe;

use crate::command::{AppCommand, CommandSender};
use crate::i18n;
use crate::i18n::Arg;
use crate::snapshot::{FailedRead, Reading};

/// Keeps the bar's room at the window's bottom while there is a dry run
/// or a failed part. Returns where [`fill`] draws it.
///
/// The room comes first, so the page's panels fit above it, and the bar
/// draws last, so the keyboard meets it after the page.
pub fn show(ui: &mut Ui, reading: &Reading, mode: Mode) -> Option<Rect> {
    let dry_run = mode.is_dry_run();
    if !bar_shows(reading, dry_run) {
        return None;
    }

    let padding = ui.tokens().sizes.status_bar_padding;
    let tokens = tokens::current(ui.ctx());

    let height = ui
        .data(|data| data.get_temp::<f32>(height_id()))
        .unwrap_or_else(|| ui.spacing().interact_size.y + padding.sum().y);

    // Over the live blur the bar is frosted, as the Console sheet is, so the
    // page under it blurs. The blur is painted before the panel's own fill.
    let frosted = toolbox_gui_kit::frost::live(ui.ctx());
    let area = ui.available_rect_before_wrap();
    let strip = Rect::from_min_max(
        pos2(area.left(), area.bottom() - height),
        area.right_bottom(),
    );

    let fill = if toolbox_gui_kit::theme::translucent(ui.ctx()) {
        tokens.band
    } else if frosted {
        tokens.frost
    } else {
        tokens.pane
    };

    if frosted {
        toolbox_gui_kit::frost::backdrop(ui.painter(), strip, CornerRadius::ZERO);
    }

    let frame = Frame::new().fill(fill).inner_margin(padding);

    let shown = Panel::bottom("status")
        .show_separator_line(false)
        .frame(frame)
        .exact_size(height)
        .show(ui, |ui| ui.max_rect());

    // The pane fill alone hardly parts the bar from a page of the same
    // shade in light.
    let bar = shown.response.rect;
    let outline = egui::Stroke::new(1.0, ui.tokens().rule);
    ui.painter().hline(bar.x_range(), bar.top() + 0.5, outline);
    Some(shown.inner)
}

/// The bar's messages and its Read again in `rect`, which [`show`] kept.
pub fn fill(ui: &mut Ui, rect: Rect, reading: &Reading, mode: Mode, commands: &CommandSender) {
    let dry_run = mode.is_dry_run();
    let failed = failed(reading);
    let padding = ui.tokens().sizes.status_bar_padding;
    let mut bar = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    let drawn = bar
        .horizontal(|row| {
            let ctx = row.ctx().clone();
            let mut said = dry_run;
            if dry_run {
                message(row, &i18n::text(&ctx, "status-dry-run"));
            }

            if reading.is_loading() {
                part(row, &mut said);
                row.loading(&i18n::text(&ctx, "status-reading"));
            }

            if !failed.is_empty() {
                part(row, &mut said);
                message(row, &failed_text(&ctx, failed));
                reasons_button(row, failed);
                let again = i18n::text(&ctx, "common-read-again");
                if icon_button(row, icons::VIEW_REFRESH, &again).clicked() {
                    commands.send(AppCommand::ReadSystem);
                }
            }
        })
        .response
        .rect;

    let height = drawn.height() + padding.sum().y;
    let kept = ui.data(|data| data.get_temp::<f32>(height_id()));
    if kept.is_none_or(|kept| (kept - height).abs() > 0.5) {
        ui.data_mut(|data| data.insert_temp(height_id(), height));
        ui.ctx()
            .request_discard("the status bar changed its height");
    }
}

/// A message the bar holds a while, led by its tile, so it reads as a
/// state without its colour.
fn message(ui: &mut Ui, text: &str) {
    status_tile(ui, TileSize::Line, Tone::Warning);
    let colour = ui.tokens().text;
    ui.label(egui::RichText::new(text).color(colour));
}

fn height_id() -> Id {
    Id::unique("toolbox_gui::status_bar::height")
}

/// A thin upright line before a message that follows another, so two
/// messages do not read as one sentence.
fn part(ui: &mut Ui, said: &mut bool) {
    if !std::mem::replace(said, true) {
        return;
    }

    let outline = tokens::current(ui.ctx()).rule;
    let height = ui.text_style_height(&egui::TextStyle::Body);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, height), egui::Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        egui::Stroke::new(1.0, outline),
    );
}

/// The bar shows for a dry run or a part that could not be read.
fn bar_shows(reading: &Reading, dry_run: bool) -> bool {
    dry_run || !failed(reading).is_empty()
}

/// A read in progress the bar does not show, as it is away. The sidebar
/// shows it then.
pub fn reads_apart(reading: &Reading, mode: Mode) -> bool {
    reading.is_loading() && !bar_shows(reading, mode.is_dry_run())
}

/// A line with a spinner over the bottom left of `page`, which moves
/// nothing.
pub fn note(ctx: &Context, page: Rect, text: &str) {
    let tokens = tokens::current(ctx);
    let inset = tokens.sizes.corner_inset;
    let glass = Glass::new(
        Frame::new()
            .corner_radius(RADIUS)
            .inner_margin(tokens.sizes.status_bar_padding),
        toolbox_gui_kit::frost::raised(ctx),
    );

    Area::new(Id::unique("toolbox_gui::status_bar::note"))
        .order(Order::Foreground)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(page.left_bottom() + egui::vec2(inset.x, -inset.y))
        .interactable(false)
        .show(ctx, |ui| {
            glass.show(ui, |ui| {
                ui.loading(text);
            });
        });
}

fn failed(reading: &Reading) -> &[FailedRead] {
    reading
        .get()
        .map_or(&[][..], |probed| probed.failed.as_slice())
}

fn part_name(ctx: &Context, probe: Probe) -> String {
    i18n::text(ctx, &format!("probe-{}", probe.as_str())).into_owned()
}

fn failed_text(ctx: &Context, failed: &[FailedRead]) -> String {
    let parts = failed
        .iter()
        .map(|read| part_name(ctx, read.probe))
        .collect::<Vec<_>>()
        .join(", ");

    i18n::text_with(ctx, "status-probes-failed", &[("parts", Arg::from(&parts))]).into_owned()
}

/// The reasons are system messages, too long for the bar. The button's
/// name carries them, so a screen reader reads them on focus, and the
/// tooltip shows them on hover and on keyboard focus.
fn reasons_button(ui: &mut Ui, failed: &[FailedRead]) {
    let ctx = ui.ctx().clone();
    let reasons = failed
        .iter()
        .map(|read| format!("{}: {}", part_name(&ctx, read.probe), read.reason))
        .collect::<Vec<_>>()
        .join("\n");

    let alt = i18n::text_with(
        &ctx,
        "status-probes-reasons",
        &[("reasons", Arg::from(&reasons))],
    );

    let button = icon_button(ui, icons::HELP_ABOUT, &alt);

    if button.has_focus() {
        button.show_tooltip_text(alt.as_ref());
    }
}
