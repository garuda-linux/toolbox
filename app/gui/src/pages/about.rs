//! Every address is fixed here, so nothing from a catalog or a program
//! ever reaches the browser.

#[cfg(test)]
mod tests;

mod exorcist;

use egui::Ui;
use egui::accesskit::Role;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{PropertyTexts, card, link_row, link_tile, property_list};

use super::header::title_header;
use super::{PageContext, PageView};
use crate::pages::heading;
use crate::{i18n, links};

const BUGS: &str = "https://gitlab.com/garuda-linux/applications/toolbox/-/issues";
pub const FORUM: &str = "https://forum.garudalinux.org/";
pub const GARUDA_WIKI: &str = "https://wiki.garudalinux.org/";
pub const ARCH_WIKI: &str = "https://wiki.archlinux.org/";
const WEBSITE: &str = "https://garudalinux.org";

/// The Help card: text id, icon and address. The forum is under
/// Community.
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
        let version = context.version;
        let _ui = tracing::trace_span!("about ui").entered();
        let called = &mut self.exorcist;
        if ui.available_width() >= SIDE_BY_SIDE {
            // Both cards take the taller one's height of the last pass, so
            // the pair ends on one line.
            let id = ui.make_persistent_id("about-top-cards");
            let tallest = ui.data(|data| data.get_temp::<f32>(id)).unwrap_or(0.0);
            let drawn = ui.columns(2, |columns| {
                let [left, right] = columns else {
                    return 0.0;
                };

                // The scope measures the content alone, so the pair also
                // shrinks again, after a language change for example.
                let facts = card(left, |column| {
                    column.set_min_height(tallest);
                    column
                        .scope(|column| facts(column, version))
                        .response
                        .rect
                        .height()
                });

                let help = card(right, |column| {
                    column.set_min_height(tallest);
                    column
                        .scope(|column| help(column, called))
                        .response
                        .rect
                        .height()
                });

                facts.inner.max(help.inner)
            });

            if (drawn - tallest).abs() > 0.5 {
                ui.data_mut(|data| data.insert_temp(id, drawn));
                ui.ctx().request_repaint();
            }
        } else {
            card(ui, |ui| facts(ui, version));
            ui.add_space(ui.tokens().sizes.card_gap);
            card(ui, |ui| help(ui, called));
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

/// The version and the addresses can be copied.
fn facts(ui: &mut Ui, version_value: &str) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id);
    let copy = |label: &str| i18n::text_with(&ctx, "about-copy", &[("label", label)]).into_owned();
    let copied = text("about-copied");
    let texts = PropertyTexts {
        copy: &copy,
        copied: &copied,
    };

    let (version, licence, made_by, bugs, website) = (
        text("about-version"),
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
            list.row_copied(&version, version_value);
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
    if ui.link(text).clicked() {
        links::open(ui.ctx(), url);
    }
}

fn help(ui: &mut Ui, called: &mut bool) {
    let ctx = ui.ctx().clone();
    let padding = ui.tokens().sizes.list_padding;
    egui::Frame::new().inner_margin(padding).show(ui, |ui| {
        for (id, icon, url) in HELP {
            if link_row(ui, icon, &i18n::text(&ctx, id), Role::Link).clicked() {
                links::open(&ctx, url);
            }
        }

        let exorcist = i18n::text(&ctx, "about-exorcist");
        if link_row(ui, icons::DRAW_CROSS, &exorcist, Role::Button).clicked() {
            *called = true;
        }
    });
}

fn web(ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::INTERNET_WEB_BROWSER, "about-web");
    tiles(ui, None, |ui, width| {
        for (id, icon, url) in WEB {
            let title = i18n::text(&ctx, id);
            let description = i18n::text(&ctx, &format!("{id}-description"));
            if link_tile(ui, icon, &title, &description, width).clicked() {
                links::open(&ctx, url);
            }
        }
    });
}

fn community(ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::USER_OTHERS, "about-community");
    tiles(ui, None, |ui, width| {
        for (id, url) in COMMUNITY {
            let title = i18n::text(&ctx, id);
            let description = i18n::text(&ctx, &format!("{id}-description"));
            if link_tile(ui, icons::DIALOG_MESSAGES, &title, &description, width).clicked() {
                links::open(&ctx, url);
            }
        }
    });
}

/// `body` gets the width of one tile.
pub fn tiles(ui: &mut Ui, most: Option<u8>, body: impl FnOnce(&mut Ui, f32)) {
    let gap = ui.tokens().sizes.card_gap;
    let room = ui.available_width();
    let fit = ((room + gap) / (TILE_WIDTH + gap)).floor();
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
