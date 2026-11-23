//! Every address is fixed here, so nothing from a catalog or a program
//! ever reaches the browser.

#[cfg(test)]
mod tests;

mod exorcist;
mod hero;

use egui::Ui;
use egui::accesskit::Role;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{
    PropertyTexts, card, link_row_at_least, link_tile, property_list, property_text_rise, text_link,
};

use super::header::title_header;
use super::{PageContext, PageView};
use crate::i18n::Arg;
use crate::pages::heading;
use crate::{i18n, links};

const BUGS: &str = "https://gitlab.com/garuda-linux/applications/toolbox/-/issues";
pub const FORUM: &str = "https://forum.garudalinux.org/";
pub const GARUDA_WIKI: &str = "https://wiki.garudalinux.org/";
pub const ARCH_WIKI: &str = "https://wiki.archlinux.org/";
const WEBSITE: &str = "https://garudalinux.org";

/// The forum is under Community.
const HELP: [(&str, Icon, &str); 3] = [
    ("about-help-wiki", icons::DOCUMENTATION, GARUDA_WIKI),
    ("about-help-arch-wiki", icons::DOCUMENTATION, ARCH_WIKI),
    (
        "about-help-status",
        icons::NETWORK_CONNECT,
        "https://status.garudalinux.org",
    ),
];

/// Garuda Linux on the web: text id (its description is `<id>-description`),
/// icon and address.
const WEB: [(&str, Icon, &str); 6] = [
    (
        "about-web-start",
        icons::INTERNET_WEB_BROWSER,
        "https://start.garudalinux.org",
    ),
    (
        "about-web-gitlab",
        icons::VCS_DIFF,
        "https://gitlab.com/garuda-linux",
    ),
    (
        "about-web-chaotic",
        icons::PACKAGE,
        "https://aur.chaotic.cx",
    ),
    (
        "about-web-searx",
        icons::GLOBE,
        "https://searx.garudalinux.org",
    ),
    (
        "about-web-vaultwarden",
        icons::SECURITY_HIGH,
        "https://bitwarden.garudalinux.org",
    ),
    (
        "about-web-donate",
        icons::GAMES_HIGHSCORES,
        "https://garudalinux.org/donate",
    ),
];

/// The community's places: text id (its description is
/// `<id>-description`) and address.
const COMMUNITY: [(&str, &str); 5] = [
    ("about-community-forum", FORUM),
    (
        "about-community-telegram",
        "https://garudalinux.org/telegram",
    ),
    ("about-community-discord", "https://garudalinux.org/discord"),
    ("about-community-mastodon", "https://social.garudalinux.org"),
    (
        "about-community-bluesky",
        "https://bsky.app/profile/garudalinux.org",
    ),
];

const SIDE_BY_SIDE: f32 = 640.0;
const TILE_WIDTH: f32 = 280.0;
/// The taller top card's height of the last pass. One About page shows at a time.
const TOP_CARDS: &str = "toolbox_gui::about::top_cards";

#[derive(Clone, Copy, Debug, Default)]
pub struct AboutPage {
    exorcist: bool,
}

impl PageView for AboutPage {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        title_header(ui, "page-about", "page-about-subtitle");
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("about ui").entered();
        hero::show(ui, context);
        ui.add_space(ui.tokens().sizes.section_gap);

        let called = &mut self.exorcist;
        if ui.available_width() >= SIDE_BY_SIDE {
            // Both cards take the taller one's height of the last pass, so
            // the pair ends on one line.
            let id = egui::Id::unique(TOP_CARDS);
            let tallest = ui.data(|data| data.get_temp::<f32>(id)).unwrap_or(0.0);
            let drawn = ui.columns(2, |columns| {
                let [left, right] = columns else {
                    return 0.0;
                };

                // The scope measures the content alone, so the pair also
                // shrinks again, after a language change for example.
                let facts = card(left, |column| {
                    column.set_min_height(tallest);
                    column.scope(facts).response.rect.height()
                });

                let help = card(right, |column| {
                    column.set_min_height(tallest);
                    column
                        .scope(|column| help(column, called, tallest))
                        .response
                        .rect
                        .height()
                });

                facts.inner.max(help.inner)
            });

            // Any change counts: at two pixels per point one pixel is half a
            // point. The measure ignores the minimum height, so the next
            // pass settles.
            if (drawn - tallest).abs() > f32::EPSILON {
                ui.data_mut(|data| data.insert_temp(id, drawn));
                ui.ctx().request_repaint();
            }
        } else {
            card(ui, facts);
            ui.add_space(ui.tokens().sizes.card_gap);
            card(ui, |ui| help(ui, called, 0.0));
        }

        ui.add_space(ui.tokens().sizes.section_gap);
        web(ui);
        ui.add_space(ui.tokens().sizes.section_gap);
        community(ui);

        if self.exorcist {
            self.exorcist = exorcist::show(ui.ctx());
        }
    }
}

/// The credits and the legal facts. The addresses can be copied, the
/// version stands in the hero.
fn facts(ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id);
    let copy = |label: &str| {
        i18n::text_with(&ctx, "about-copy", &[("label", Arg::from(label))]).into_owned()
    };

    let copied = text("about-copied");

    let texts = PropertyTexts {
        copy: &copy,
        copied: &copied,
    };

    let (licence, made_by, bugs, website) = (
        text("about-licence"),
        text("about-made-by"),
        text("about-bugs"),
        text("about-website"),
    );

    let (licence_value, made_by_value) = (text("about-licence-value"), text("about-made-by-value"));
    let tracker = text("about-bugs-link");
    let padding = ui.tokens().sizes.card_padding;
    egui::Frame::new().inner_margin(padding).show(ui, |ui| {
        property_list(ui, texts, |list| {
            list.row(&licence, &licence_value);
            list.row(&made_by, &made_by_value);
            list.row_with(&bugs, Some(BUGS), |row| address(row, &tracker, BUGS));
            list.row_with(&website, Some(WEBSITE), |row| {
                address(row, "garudalinux.org", WEBSITE);
            });
        });
    });
}

fn address(ui: &mut Ui, text: &str, url: &str) {
    if text_link(ui, text, text).clicked() {
        links::open(ui.ctx(), url);
    }
}

/// The help rows. Beside the facts they share `tallest`, the card's height,
/// so they space out as the facts do instead of leaving a band below.
///
/// A fact row is its line plus half the gap between facts on either side.
/// The help rows take that pitch, start half a gap above the first fact and
/// rise as far as a fact's text sits above its row middle, so each lines up
/// with the fact beside it.
fn help(ui: &mut Ui, called: &mut bool, tallest: f32) {
    let ctx = ui.ctx().clone();
    let sizes = ui.tokens().sizes;
    let edge = f32::from(sizes.card_padding.top) - sizes.control_gap / 2.0;
    let rise = property_text_rise(ui);
    let rows = f32::from(u8::try_from(HELP.len() + 1).unwrap_or(u8::MAX));
    let least = edge.mul_add(-2.0, tallest) / rows;
    let margin = egui::Margin {
        top: 0,
        bottom: 0,
        ..sizes.list_padding
    };

    egui::Frame::new().inner_margin(margin).show(ui, |ui| {
        ui.add_space(edge - rise);
        for (id, icon, url) in HELP {
            let text = i18n::text(&ctx, id);
            if link_row_at_least(ui, icon, &text, Role::Link, least).clicked() {
                links::open(&ctx, url);
            }
        }

        let exorcist = i18n::text(&ctx, "about-exorcist");
        if link_row_at_least(ui, icons::DRAW_CROSS, &exorcist, Role::Button, least).clicked() {
            *called = true;
        }

        ui.add_space(edge + rise);
    });
}

fn web(ui: &mut Ui) {
    link_tiles(ui, "about-web", WEB);
}

fn community(ui: &mut Ui) {
    link_tiles(
        ui,
        "about-community",
        COMMUNITY.map(|(id, url)| (id, icons::DIALOG_MESSAGES, url)),
    );
}

fn link_tiles(
    ui: &mut Ui,
    heading_id: &str,
    entries: impl IntoIterator<Item = (&'static str, Icon, &'static str)>,
) {
    let ctx = ui.ctx().clone();
    heading(ui, heading_id);
    tiles(ui, None, |ui, width| {
        for (id, icon, url) in entries {
            let title = i18n::text(&ctx, id);
            let description = i18n::text(&ctx, &format!("{id}-description"));
            if link_tile(ui, icon, &title, &description, width).clicked() {
                links::open(&ctx, url);
            }
        }
    });
}

/// `body` gets the width of one tile.
pub fn tiles(ui: &mut Ui, most: Option<u8>, body: impl FnOnce(&mut Ui, f32)) {
    tiles_at_least(ui, most, TILE_WIDTH, body);
}

/// [`tiles`] with tiles no narrower than `least`.
pub fn tiles_at_least(ui: &mut Ui, most: Option<u8>, least: f32, body: impl FnOnce(&mut Ui, f32)) {
    let gap = ui.tokens().sizes.card_gap;
    let room = ui.available_width();
    let fit = ((room + gap) / (least + gap)).floor();
    let columns = fit.clamp(1.0, most.map_or(f32::INFINITY, f32::from));

    // Down to whole pixels: a hair over the room makes the wrapping line
    // push the last tile of each line onto its own.
    let pixels = ui.ctx().pixels_per_point();
    let width = (gap.mul_add(1.0 - columns, room) / columns * pixels).floor() / pixels;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(gap, gap);
        body(ui, width);
    });
}
