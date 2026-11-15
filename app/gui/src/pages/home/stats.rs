//! The stat tiles under the greeting. Each shows one figure with its mark,
//! green while it is fine, and opens the Maintenance tab that acts on it.

use egui::accesskit::Role;
use egui::{Align, CursorIcon, Frame, Layout, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo};
use toolbox_cli::output::size;
use toolbox_gui_kit::fonts::{SECTION, SMALL, Weight};
use toolbox_gui_kit::theme::CARD_RADIUS;
use toolbox_gui_kit::widgets::{TileSize, Tone, focus_ring, rich, status_tile};
use toolbox_gui_kit::{Loadable, UiExt as _, tokens};
use toolbox_services::home::Overview;
use toolbox_services::home::stats::{Figure, Standing, Stat, Tile, tiles};
use toolbox_system::HealthLevel;

use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::maintenance::Tab;

/// Below this a tile cuts its figure, so the four take two lines.
const MIN_TILE_WIDTH: f32 = 180.0;

/// The tiles of the status read and the facts, unknown until each is read.
pub fn show(ui: &mut Ui, context: &PageContext<'_>, overview: Option<&Overview<'_>>) {
    let facts = context.home.facts.and_then(Loadable::get);
    let shown = tiles(overview, facts);

    let gap = ui.tokens().sizes.card_gap;
    let room = ui.available_width();
    let columns: u8 = if 4.0_f32.mul_add(MIN_TILE_WIDTH, 3.0 * gap) <= room {
        4
    } else {
        2
    };

    let count = f32::from(columns);
    let pixels = ui.ctx().pixels_per_point();
    let width = (gap.mul_add(1.0 - count, room) / count * pixels).floor() / pixels;

    // One row per line of tiles, each row top aligned, so a tile never
    // sits lower than its neighbour.
    ui.spacing_mut().item_spacing.y = gap;
    for line in shown.chunks(usize::from(columns)) {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for tile in line {
                if stat_tile(ui, tile, width).clicked() {
                    let tab = match tile.stat {
                        Stat::Health => Tab::Repairs,
                        Stat::Updates | Stat::Disk | Stat::LastUpdate => Tab::Common,
                    };

                    context.commands.send(MaintenanceCommand::Reveal(tab));
                }
            }
        });
    }
}

/// A link named by the tile's name and figure.
fn stat_tile(ui: &mut Ui, tile: &Tile, width: f32) -> Response {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let name = i18n::text(&ctx, &format!("home-stat-{}", tile.stat.key()));
    let value = figure(&ctx, tile);
    let tone = match tile.standing {
        Standing::Fine => Tone::Success,
        Standing::Attention => Tone::Warning,
        Standing::Unknown => Tone::Neutral,
    };

    // White on near white needs a rim, a dark card stands off by its fill.
    let rim = if ui.visuals().dark_mode {
        Stroke::NONE
    } else {
        Stroke::new(1.0, tokens.outline)
    };

    let response = ui
        .scope_builder(
            UiBuilder::new()
                .sense(Sense::click())
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                let last = ui.response();
                let fill = if last.is_pointer_button_down_on() {
                    tokens.pressed
                } else if last.hovered() {
                    tokens.hover
                } else {
                    tokens.card
                };

                ui.set_width(width);
                Frame::new()
                    .fill(fill)
                    .stroke(rim)
                    .corner_radius(CARD_RADIUS)
                    .inner_margin(tokens.sizes.card_padding)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
                        ui.horizontal(|ui| {
                            status_tile(ui, TileSize::Line, tone);
                            ui.label(rich(ui, &name, Weight::Medium, SMALL, tokens.secondary));
                        });

                        ui.add(
                            egui::Label::new(rich(ui, &value, Weight::Bold, SECTION, tokens.text))
                                .truncate(),
                        );
                    });
            },
        )
        .response;

    response.widget_info(|| WidgetInfo::labeled(Role::Link, true, format!("{name}: {value}")));
    focus_ring(ui, &response, CARD_RADIUS);
    if response.hovered() {
        ctx.set_cursor_icon(CursorIcon::PointingHand);
    }

    response
}

fn figure(ctx: &egui::Context, tile: &Tile) -> String {
    let text = match tile.figure {
        Figure::Unknown => i18n::text(ctx, "home-stat-unknown"),
        Figure::Updates(0) => i18n::text(ctx, "home-stat-updates-none"),
        Figure::Updates(count) => i18n::text_with(
            ctx,
            "home-stat-updates-count",
            &[("count", &count.to_string())],
        ),
        Figure::Free(bytes) => {
            i18n::text_with(ctx, "home-stat-disk-free", &[("free", &size(bytes))])
        }
        Figure::Health(None) => i18n::text(ctx, "home-stat-health-fine"),
        Figure::Health(Some(level)) if level >= HealthLevel::High => {
            i18n::text(ctx, "home-stat-health-errors")
        }
        Figure::Health(Some(_)) => i18n::text(ctx, "home-stat-health-warnings"),
        Figure::Days(0) => i18n::text(ctx, "home-stat-last-update-today"),
        Figure::Days(days) => i18n::text_with(
            ctx,
            "home-stat-last-update-days",
            &[("days", &days.to_string())],
        ),
    };

    text.into_owned()
}
