use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use parking_lot::Mutex;

use super::AboutPage;
use crate::command::{AppCommand, Command};
use crate::links::{Opener, install};
use crate::testing::{PageFixture, PageState};

#[derive(Debug, Default)]
struct Opened(Mutex<Vec<String>>);

impl Opener for Opened {
    fn open(&self, url: &str) -> bool {
        self.0.lock().push(url.to_owned());
        true
    }
}

fn page() -> (Harness<'static, PageState<AboutPage>>, Arc<Opened>) {
    let mut harness = PageFixture::default().harness(AboutPage::default());
    let opened = Arc::new(Opened::default());
    let fake: Arc<dyn Opener> = Arc::<Opened>::clone(&opened);
    install(&harness.ctx, fake);
    harness.run();
    (harness, opened)
}

fn wait_for_open(opened: &Opened) -> Vec<String> {
    for _ in 0..200_u16 {
        let seen = opened.0.lock().clone();
        if !seen.is_empty() {
            return seen;
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    Vec::new()
}

#[test]
fn the_version_and_licence_show() {
    let (harness, _) = page();

    assert!(harness.query_by_label(crate::FIXED_VERSION).is_some());
    assert!(harness.query_by_label("GPL 3.0 or later").is_some());
}

#[test]
fn the_hero_names_the_app_above_its_version_and_copies_the_system_details() {
    let (mut harness, _) = page();
    let name = harness.get_by_label("Garuda Toolbox").rect();
    let version = harness.get_by_label(crate::FIXED_VERSION).rect();
    let credits = harness.get_by_label("Made by the Garuda Linux team").rect();

    assert!(name.bottom() <= version.top(), "{name:?} {version:?}");
    assert!(version.bottom() <= credits.top(), "{version:?} {credits:?}");

    harness.get_by_label("Copy System Details").click();
    harness.run();

    assert!(matches!(
        harness.state().fixture.sent().as_slice(),
        [Command::App(AppCommand::Toast { text, kind: _ })]
            if text == "Copied the system details to the clipboard."
    ));
}

#[test]
fn a_help_link_opens_its_page() {
    let (mut harness, opened) = page();

    harness
        .get_by_role_and_label(Role::Link, "Garuda Linux wiki")
        .click();
    harness.run();

    assert_eq!(wait_for_open(&opened), ["https://wiki.garudalinux.org/"]);
}

fn call_the_exorcist(harness: &mut Harness<'static, PageState<AboutPage>>) {
    harness
        .get_by_role_and_label(Role::Button, "Call the Exorcist")
        .click();
    harness.run();
}

#[test]
fn the_exorcist_answers_in_a_dialog_not_a_note() {
    let (mut harness, _) = page();

    call_the_exorcist(&mut harness);

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Exorcist Called")
            .is_some()
    );
    assert!(
        harness
            .query_all_by_label_contains("special voodoo")
            .next()
            .is_some()
    );
    assert_eq!(harness.state().fixture.sent(), Vec::<Command>::new());
}

#[test]
fn escape_sends_the_exorcist_away() {
    let (mut harness, _) = page();
    call_the_exorcist(&mut harness);

    harness.key_press(egui::Key::Escape);
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Exorcist Called")
            .is_none()
    );
}

#[test]
fn the_exorcist_points_to_the_forum_and_the_tracker() {
    for (button, url) in [
        ("Forum", "https://forum.garudalinux.org/"),
        ("Issue tracker", super::BUGS),
    ] {
        let (mut harness, opened) = page();
        call_the_exorcist(&mut harness);

        harness.get_by_role_and_label(Role::Button, button).click();
        harness.run();

        assert_eq!(wait_for_open(&opened), [url]);
    }
}

#[test]
fn the_web_and_community_lists_show() {
    let (harness, _) = page();

    for name in ["GitLab", "Chaotic-AUR", "Donate", "Telegram", "Bluesky"] {
        assert!(harness.query_all_by_label(name).next().is_some(), "{name}");
    }
}

#[test]
fn every_community_place_says_what_it_is_for() {
    let (harness, _) = page();

    let descriptions: std::collections::BTreeSet<_> =
        ["Forum", "Telegram", "Discord", "Mastodon", "Bluesky"]
            .map(|name| {
                harness
                    .get_by_role_and_label(Role::Link, name)
                    .accesskit_node()
                    .description()
            })
            .into_iter()
            .collect();

    assert_eq!(
        descriptions.len(),
        5,
        "shared descriptions: {descriptions:?}"
    );
}

#[test]
fn every_place_is_listed_once_and_the_start_page_is_on_the_web() {
    let (harness, _) = page();

    let forums = harness.query_all_by_label("Forum").count();
    assert_eq!(forums, 1, "the forum is listed {forums} times");
    let start = harness.get_by_role_and_label(Role::Link, "Start page");
    assert_eq!(
        start.accesskit_node().description().as_deref(),
        Some("The start page for your browser")
    );
}

/// The outlines of the two cards side by side: the facts, then the help.
fn top_cards(harness: &Harness<'static, PageState<AboutPage>>) -> (egui::Rect, egui::Rect) {
    let fill = toolbox_gui_kit::tokens::current(&harness.ctx).card_glass;
    let mut cards = toolbox_gui_kit::testing::filled_rects(harness, fill);
    cards.sort_by(|first, second| first.top().total_cmp(&second.top()));
    let [first, second, ..] = cards[..] else {
        panic!("two cards, found {cards:?}");
    };

    if first.left() < second.left() {
        (first, second)
    } else {
        (second, first)
    }
}

fn assert_one_line(harness: &Harness<'static, PageState<AboutPage>>, case: &str) {
    let (facts, help) = top_cards(harness);

    assert!(facts.right() < help.left(), "{case}: {facts:?} {help:?}");
    assert!(
        (facts.top() - help.top()).abs() < 0.01 && (facts.bottom() - help.bottom()).abs() < 0.01,
        "{case}: {facts:?} {help:?}, bottoms {} apart",
        facts.bottom() - help.bottom()
    );
}

#[test]
fn the_two_top_cards_end_on_one_line() {
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        for language in ["en-US", "de-DE"] {
            let (mut harness, _opened) = page();
            harness.ctx.set_theme(theme);
            crate::i18n::set_language(&harness.ctx, &language.parse().expect("a language"));
            harness.run();

            assert_one_line(&harness, &format!("{theme:?} {language}"));
        }
    }
}

/// At two pixels per point one pixel is half a point, so a row that
/// grows by a pixel, with another font for example, leaves the last
/// measure a fraction short.
#[test]
fn the_two_top_cards_meet_again_after_a_fraction_of_a_point() {
    let (mut harness, _opened) = page();
    let id = egui::Id::unique(super::TOP_CARDS);
    let measured = harness.ctx.data(|data| data.get_temp::<f32>(id));
    let short = measured.expect("a measure of the taller card") - 0.25;
    harness.ctx.data_mut(|data| data.insert_temp(id, short));
    harness.run();

    assert_one_line(&harness, "a measure 0.25 short");
}

#[test]
fn only_technical_facts_offer_a_copy_button() {
    let (mut harness, _) = page();
    harness.run();

    let copy = |label: &str| {
        let named = format!("Copy {label}");
        harness.query_by_label(&named).is_some()
    };

    assert!(
        harness.query_by_label("Copy System Details").is_some(),
        "the version and the system are what a bug report needs"
    );
    assert!(!copy("Made by"), "a name is nothing to paste");
    assert!(!copy("License"), "the license is nothing to paste");
}

#[test]
fn tiles_fill_every_column_of_a_line_at_any_width() {
    for width in 880_u16..=920 {
        let mut harness =
            toolbox_gui_kit::testing::harness(f32::from(width), 300.0).build_ui(|ui| {
                super::tiles(ui, None, |ui, tile| {
                    for title in ["One", "Two", "Three"] {
                        toolbox_gui_kit::widgets::link_tile(
                            ui,
                            toolbox_gui_kit::icons::USER_OTHERS,
                            title,
                            "",
                            tile,
                        );
                    }
                });
            });

        harness.run();
        let top = |title: &str| harness.get_by_label(title).rect().top();
        assert!(
            (top("One") - top("Three")).abs() < 0.5,
            "at {width}: the third tile wrapped onto its own line"
        );
    }
}

#[test]
fn retired_services_are_no_longer_listed() {
    let (harness, _) = page();

    for name in ["Whoogle", "IRC"] {
        assert!(harness.query_all_by_label(name).next().is_none(), "{name}");
    }
}

#[test]
fn each_help_row_lines_up_with_a_fact() {
    let (mut harness, _opened) = page();
    harness.run();

    let centre = |label: &str| {
        let text = toolbox_gui_kit::testing::painted_text(&harness, label).expect(label);
        text.pos.y + text.galley.rect.center().y
    };

    for (fact, help) in [
        ("License", "Garuda Linux wiki"),
        ("Made by", "Arch Linux wiki"),
        ("Bug reports", "Service status"),
        ("Website", "Call the Exorcist"),
    ] {
        assert!(
            (centre(fact) - centre(help)).abs() < 1.0,
            "{fact} at {}, {help} at {}",
            centre(fact),
            centre(help)
        );
    }
}
