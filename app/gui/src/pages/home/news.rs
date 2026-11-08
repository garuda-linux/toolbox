//! Garuda's and Arch's news in one list, newest first. Titles are the
//! sites' words, shown as plain text. A title opens its post only when the
//! link leads into its source's site, which the news crate already
//! checked and this page checks again.

use egui::{Align, Label, Layout, Ui, vec2};
use toolbox_gui_kit::fonts::{BODY, SECTION, SMALL, Weight};
use toolbox_gui_kit::widgets::{Tone, card, card_row, rich, split_row, tag, text_link};
use toolbox_gui_kit::{Loadable, UiExt as _, tokens};
use toolbox_news::{News, Post, Source};

use crate::pages::PageContext;
use crate::pages::packages::day;
use crate::{i18n, links};

/// Home shows the newest few, the sites hold the rest.
const SHOWN: usize = 4;

/// Room for a date as `2026-11-04`.
const DATE_WIDTH: f32 = 96.0;

/// Room for the longer source tag, Garuda.
const SOURCE_WIDTH: f32 = 72.0;

/// Room for the two links beside the heading.
const LINKS_WIDTH: f32 = 240.0;

/// Nothing while the first read runs or when no copy exists: the news
/// is a nicety, Home does not warn about it.
pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(news) = context.home.news.and_then(Loadable::get) else {
        return;
    };

    if news.posts.is_empty() {
        return;
    }

    ui.add_space(ui.tokens().sizes.block_gap);
    heading(ui);
    ui.add_space(ui.tokens().sizes.card_gap);
    posts(ui, news);
}

fn heading(ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    split_row(
        ui,
        LINKS_WIDTH,
        |ui| {
            let title = i18n::text(&ctx, "home-news");
            ui.label(rich(ui, &title, Weight::Bold, SECTION, tokens.text));
        },
        |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Right to left, so Garuda's link stands first when read.
                for source in Source::ALL.into_iter().rev() {
                    let text = i18n::text(&ctx, &format!("home-news-{}-page", source.key()));
                    if text_link(ui, &text, &text).clicked() {
                        links::open(&ctx, source.page());
                    }
                }
            });
        },
    );
}

fn posts(ui: &mut Ui, news: &News) {
    card(ui, |ui| {
        for (index, post) in news.posts.iter().take(SHOWN).enumerate() {
            card_row(ui, index == 0, |ui| row(ui, post));
        }
    });
}

fn row(ui: &mut Ui, post: &Post) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    ui.horizontal(|ui| {
        let date = post.published.map(day).unwrap_or_default();
        let height = ui.spacing().interact_size.y;
        let layout = Layout::left_to_right(Align::Center);
        ui.allocate_ui_with_layout(vec2(DATE_WIDTH, height), layout, |ui| {
            ui.set_min_width(DATE_WIDTH);
            ui.label(rich(ui, &date, Weight::Regular, SMALL, tokens.secondary));
        });

        // A cell of its own, so the titles line up whatever the source.
        let source = i18n::text(&ctx, &format!("home-news-{}", post.source.key()));
        ui.allocate_ui_with_layout(vec2(SOURCE_WIDTH, height), layout, |ui| {
            ui.set_min_width(SOURCE_WIDTH);
            tag(ui, &source, Tone::Neutral);
        });

        let link = post
            .link
            .as_deref()
            .filter(|link| link.starts_with(post.source.post_prefix()));

        if let Some(link) = link {
            let name = i18n::text_with(&ctx, "home-news-open", &[("title", &post.title)]);
            if text_link(ui, &post.title, &name).clicked() {
                links::open(&ctx, link);
            }
        } else {
            let title = rich(ui, &post.title, Weight::Medium, BODY, tokens.text);
            ui.add(Label::new(title).wrap());
        }

        if post.manual {
            tag(ui, &i18n::text(&ctx, "home-news-manual"), Tone::Warning);
        }
    });
}
