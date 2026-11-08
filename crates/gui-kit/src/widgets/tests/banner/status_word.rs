use egui::accesskit::{Live, Role};
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use crate::testing::{circle_fills, harness, text_colour, textured_rects};
use crate::tokens;
use crate::widgets::{Tone, status_word};

#[test]
fn a_status_word_is_read_when_it_changes() {
    for (tone, live) in [
        (Tone::Neutral, Live::Polite),
        (Tone::Accent, Live::Polite),
        (Tone::Success, Live::Polite),
        (Tone::Error, Live::Assertive),
    ] {
        let harness = harness(400.0, 100.0).build_ui(move |ui| {
            _ = status_word(ui, "All changes applied", tone);
        });

        let word = harness.get_by_label("All changes applied");
        assert_eq!(word.accesskit_node().live(), live, "{tone:?}");
    }
}

#[test]
fn a_failed_status_word_is_an_alert() {
    let harness = harness(400.0, 100.0).build_ui(|ui| {
        _ = status_word(ui, "Failed", Tone::Error);
    });

    let word = harness.get_by_label("Failed");
    assert_eq!(word.accesskit_node().role(), Role::Alert);
}

#[test]
fn an_ended_status_shows_its_mark_and_the_word_in_the_secondary_colour() {
    for tone in [Tone::Success, Tone::Error] {
        let mut harness = harness(400.0, 100.0).build_ui(move |ui| {
            _ = status_word(ui, "Done", tone);
        });

        harness.run();

        let secondary = tokens::current(&harness.ctx).secondary;
        assert_eq!(textured_rects(&harness).len(), 1, "{tone:?}");
        assert!(circle_fills(&harness).is_empty(), "{tone:?}");
        assert_eq!(text_colour(&harness, "Done"), Some(secondary));
    }
}

#[test]
fn a_running_or_idle_status_shows_a_dot_and_no_mark() {
    for tone in [Tone::Accent, Tone::Neutral] {
        let mut harness = harness(400.0, 100.0).build_ui(move |ui| {
            _ = status_word(ui, "Running", tone);
        });

        harness.run();

        let secondary = tokens::current(&harness.ctx).secondary;
        assert!(textured_rects(&harness).is_empty(), "{tone:?}");
        assert_eq!(circle_fills(&harness).len(), 1, "{tone:?}");
        assert_eq!(text_colour(&harness, "Running"), Some(secondary));
    }
}

#[test]
fn a_mark_sits_on_the_middle_of_its_word() {
    let mut harness = harness(400.0, 100.0).build_ui(|ui| {
        _ = status_word(ui, "Done", Tone::Success);
    });

    harness.run();

    let mark = textured_rects(&harness)[0];
    let word = harness.get_by_label("Done").rect();
    assert!(
        (mark.center().y - word.center().y).abs() < 1.0,
        "{mark:?} {word:?}"
    );
}
