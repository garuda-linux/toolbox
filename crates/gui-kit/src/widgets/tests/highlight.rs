use super::super::{highlighted, matches};
use crate::fonts::{self, BODY, Weight};
use crate::testing::harness;
use crate::tokens;

#[test]
fn matches_ignore_case() {
    assert_eq!(
        matches("PipeWire and pipewire-pulse", "pipe"),
        [0..4, 13..17]
    );
}

#[test]
fn an_empty_query_matches_nothing() {
    assert_eq!(
        matches("PipeWire", "  "),
        Vec::<std::ops::Range<usize>>::new()
    );
}

#[test]
fn a_match_maps_back_across_letters_whose_lowercase_is_longer() {
    // İ lowercases to two characters, so the lowercase text is longer.
    let text = "İstanbul Steam";

    let found = matches(text, "steam");

    assert_eq!(found.len(), 1);
    assert_eq!(text.get(found[0].clone()), Some("Steam"));
}

/// The search fields compare both lower case sigmas as one letter
/// (`toolbox_core::features::search::Needle`), so the wash must too.
#[test]
fn the_two_lower_case_sigmas_match_each_other() {
    let text = "Λόγος λόγοσ";

    let marked: Vec<&str> = matches(text, "ΛΌΓΟΣ")
        .into_iter()
        .filter_map(|range| text.get(range))
        .collect();

    assert_eq!(marked, ["Λόγος", "λόγοσ"]);
}

#[test]
fn the_matched_parts_get_the_accent_wash() {
    let mut harness = harness(200.0, 100.0).build_ui(|ui| {
        let font = fonts::font(ui.style(), Weight::Regular, BODY);
        let job = highlighted(
            ui,
            "Steam and steamcmd",
            "steam",
            font,
            egui::Color32::WHITE,
        );

        let soft = tokens::current(ui.ctx()).accent_soft;

        let washed: Vec<&str> = job
            .sections
            .iter()
            .filter(|section| section.format.background == soft)
            .filter_map(|section| {
                job.text
                    .get(section.byte_range.start.0..section.byte_range.end.0)
            })
            .collect();

        assert_eq!(washed, ["Steam", "steam"]);
        assert_eq!(job.text, "Steam and steamcmd");
    });

    harness.run();
}
