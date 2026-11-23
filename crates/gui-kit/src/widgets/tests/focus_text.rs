//! The label of a control that keyboard focus fills reaches 4.5:1 on the
//! fill, as the text of a focused push button does.

use egui::accesskit::Role;
use egui::{Color32, Theme, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::buttons::ButtonKind;
use super::super::pill_button;
use crate::testing::{fill_under, harness, painted_text, tab_into};
use crate::tokens::{self, Tokens, contrast};

const LABEL: &str = "Apply";
const RATIO: f32 = 4.5;

/// The label's colour, the fill under it and the tokens, with `kind`
/// reached by Tab.
fn focused(kind: ButtonKind, theme: Theme) -> (Color32, Color32, Tokens) {
    let mut harness: Harness<'static> =
        harness(300.0, 120.0).build_ui(move |ui: &mut Ui| _ = pill_button(ui, LABEL, kind));

    harness.ctx.set_theme(theme);
    harness.run();
    assert!(tab_into(&mut harness, Role::Button), "{kind:?} takes Tab");
    harness.run();

    let centre = harness.get_by_label(LABEL).rect().center();

    // The label is laid out without a colour, so it is painted in the
    // fallback colour the shape carries.
    let ink = painted_text(&harness, LABEL)
        .expect("the label is painted")
        .fallback_color;

    let fill = fill_under(&harness, centre).expect("the button is filled");
    (ink, fill, tokens::current(&harness.ctx))
}

#[test]
fn a_focused_button_label_reads_at_4_5_on_the_focus_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        for kind in [
            ButtonKind::Primary,
            ButtonKind::Soft,
            ButtonKind::Danger,
            ButtonKind::Ghost,
        ] {
            let (ink, fill, tokens) = focused(kind, theme);

            assert_eq!(fill, tokens.focus_fill, "{theme:?}: {kind:?} fill");
            let ratio = contrast(ink, fill);
            assert!(
                ratio >= RATIO,
                "{theme:?}: {kind:?} label {ink:?} {ratio:.2}:1 on {fill:?}, token {:?}",
                tokens.on_focus
            );
        }
    }
}
