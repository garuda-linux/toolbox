use egui::Theme;

use super::super::press::Pointer;
use super::super::segmented::segment_fill;
use super::super::select::option_fill;
use crate::palette::Palette;
use crate::tokens::Tokens;

fn tokens() -> Tokens {
    Tokens::derive(&Palette::fallback(Theme::Dark), Theme::Dark)
}

#[test]
fn a_pressed_surface_takes_the_step_past_its_hover() {
    let tokens = tokens();

    for rest in [tokens.card, tokens.field] {
        assert_eq!(Pointer::Rest.fill(&tokens, rest), rest);
        assert_eq!(Pointer::Hovered.fill(&tokens, rest), tokens.hover);
        assert_eq!(Pointer::Pressed.fill(&tokens, rest), tokens.pressed);
    }
}

#[test]
fn a_pressed_option_deepens_chosen_or_not() {
    let tokens = tokens();

    assert_eq!(
        option_fill(&tokens, true, Pointer::Rest),
        Some(tokens.accent_soft)
    );
    assert_eq!(option_fill(&tokens, false, Pointer::Rest), None);
    for chosen in [true, false] {
        assert_eq!(
            option_fill(&tokens, chosen, Pointer::Hovered),
            Some(tokens.hover)
        );
        assert_eq!(
            option_fill(&tokens, chosen, Pointer::Pressed),
            Some(tokens.pressed)
        );
    }
}

#[test]
fn a_pressed_segment_shows_before_it_is_chosen() {
    let tokens = tokens();

    assert_eq!(segment_fill(&tokens, false, Pointer::Rest), None);
    assert_eq!(segment_fill(&tokens, false, Pointer::Hovered), None);
    assert_eq!(
        segment_fill(&tokens, false, Pointer::Pressed),
        Some(tokens.pressed)
    );
    assert_eq!(
        segment_fill(&tokens, true, Pointer::Pressed),
        None,
        "the sliding pill shows the chosen one"
    );
}
