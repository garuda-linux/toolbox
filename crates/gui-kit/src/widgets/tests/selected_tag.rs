//! A tag on a selected row sits on the selection fill, so its words and its
//! frame take the selection's text colour.

use egui::{Theme, Ui};
use egui_kittest::Harness;

use super::super::tag_on_row;
use crate::testing::{harness, painted_text};
use crate::tokens::{self, contrast};

const WORDS: &str = "AUR";

fn shown(theme: Theme) -> Harness<'static> {
    let mut harness = harness(300.0, 120.0).build_ui(|ui: &mut Ui| _ = tag_on_row(ui, WORDS, true));

    harness.ctx.set_theme(theme);
    harness.run();
    harness
}

#[test]
fn a_tag_on_a_selected_row_is_drawn_in_the_selection_text() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = shown(theme);
        let tokens = tokens::current(&harness.ctx);
        let ink = painted_text(&harness, WORDS)
            .expect("the tag is painted")
            .fallback_color;

        assert_eq!(ink, tokens.on_tint, "{theme:?}");
        let ratio = contrast(ink, tokens.accent_tint);
        assert!(ratio >= 3.0, "{theme:?}: tag {ratio:.2}:1 on the selection");
    }
}
