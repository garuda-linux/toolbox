use egui::{Color32, Theme};

use super::super::press::{ItemLook, Pointer, item_look};
use super::super::segmented::segment_fill;
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
        assert_eq!(Pointer::Hovered.fill(&tokens, rest), tokens.button_hover);
        assert_eq!(Pointer::Pressed.fill(&tokens, rest), tokens.button_pressed);
    }
}

#[test]
fn a_pressed_tab_takes_the_hover_as_breeze_does() {
    let tokens = tokens();

    assert_eq!(
        segment_fill(&tokens, false, Pointer::Pressed),
        segment_fill(&tokens, false, Pointer::Hovered)
    );
    assert_eq!(
        segment_fill(&tokens, false, Pointer::Rest),
        Color32::TRANSPARENT
    );
}

#[test]
fn a_list_item_looks_as_darkly_draws_it() {
    let tokens = tokens();

    assert_eq!(
        item_look(&tokens, false, Pointer::Rest),
        ItemLook {
            fill: Color32::TRANSPARENT,
            frame: Color32::TRANSPARENT,
        }
    );
    for pointer in [Pointer::Hovered, Pointer::Pressed] {
        assert_eq!(
            item_look(&tokens, false, pointer),
            ItemLook {
                fill: tokens.item_hover,
                frame: Color32::TRANSPARENT,
            }
        );
    }

    assert_eq!(
        item_look(&tokens, true, Pointer::Rest),
        ItemLook {
            fill: tokens.accent_tint,
            frame: Color32::TRANSPARENT,
        }
    );
    for pointer in [Pointer::Hovered, Pointer::Pressed] {
        assert_eq!(
            item_look(&tokens, true, pointer),
            ItemLook {
                fill: tokens.tint_hover,
                frame: Color32::TRANSPARENT,
            }
        );
    }
}
