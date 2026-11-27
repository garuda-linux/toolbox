//! Darkly's tabs, switches and check boxes, checked on the pure formulas
//! in both themes.

use egui::{Color32, Rect, Theme, pos2, vec2};

use super::super::check::{box_fill, partial};
use super::super::press::Pointer;
use super::super::segmented::{corners, segment_fill, tab_fill};
use super::super::switch::{knob_colours, track_colours};
use super::darkly_buttons::tokens;
use crate::theme::{RADIUS, TAB_RADIUS};
use crate::tokens::{darkly, mix};

#[test]
fn a_tab_has_no_fill_until_it_is_chosen_or_pointed_at() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = tokens(theme);

        assert_eq!(
            segment_fill(&tokens, true, Pointer::Rest),
            tokens.input,
            "{theme:?}: the chosen tab takes the input fill"
        );
        assert_eq!(
            segment_fill(&tokens, true, Pointer::Hovered),
            tokens.input,
            "{theme:?}: a chosen tab does not take the hover"
        );
        assert_eq!(
            segment_fill(&tokens, false, Pointer::Rest),
            Color32::TRANSPARENT,
            "{theme:?}: an unchosen tab has no fill, its text stands alone"
        );
        assert_eq!(
            segment_fill(&tokens, false, Pointer::Hovered),
            tokens.item_hover,
            "{theme:?}: hover is the item hover"
        );
        assert_eq!(
            segment_fill(&tokens, false, Pointer::Pressed),
            segment_fill(&tokens, false, Pointer::Hovered),
            "{theme:?}: a press keeps the hover fill"
        );
        assert_eq!(
            tab_fill(&tokens, true, Pointer::Rest, true),
            tokens.focus_fill,
            "{theme:?}: a focused tab fills with the focus colour, chosen or not"
        );
        assert_eq!(
            tab_fill(&tokens, false, Pointer::Hovered, true),
            tokens.focus_fill,
            "{theme:?}: focus wins over the hover"
        );
        assert_eq!(
            tab_fill(&tokens, false, Pointer::Rest, false),
            Color32::TRANSPARENT
        );
    }
}

/// Every corner of a tab is rounded. The chosen one is a pill of the
/// menu radius, the unchosen ones the tab radius.
#[test]
fn every_corner_of_a_tab_is_rounded_and_the_chosen_one_is_a_pill() {
    assert_eq!(corners(true), egui::CornerRadius::same(RADIUS));
    assert_eq!(corners(false), egui::CornerRadius::same(TAB_RADIUS));
}

#[test]
fn a_switch_track_and_knob_keep_their_shape_with_darkly_highlight() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = tokens(theme);
        let (text, accent) = (tokens.text, tokens.accent_tint);
        let frame = tokens.rim;
        let track = tokens.input;

        assert_eq!(
            track_colours(&tokens, false, false),
            (track, frame),
            "{theme:?}: off"
        );
        assert_eq!(
            track_colours(&tokens, false, true),
            (track, tokens.accent),
            "{theme:?}: off and hovered"
        );
        assert_eq!(
            track_colours(&tokens, true, false),
            (track, tokens.accent),
            "{theme:?}: on, the track keeps its colour and the frame turns"
        );
        assert_eq!(
            tokens.accent_tint, accent,
            "{theme:?}: the highlight is the selection colour (Highlight), over the part the knob has passed"
        );
        let (knob, knob_frame) = (tokens.field, mix(tokens.field, text, 0.2));
        assert_eq!(knob_colours(&tokens, false), (knob, knob_frame));
        assert_eq!(knob_colours(&tokens, true), (knob, tokens.accent));
    }
}

#[test]
fn a_check_box_fills_as_darkly_draws_it() {
    for theme in [Theme::Dark, Theme::Light] {
        let tokens = tokens(theme);
        let (button, highlight) = (tokens.input, tokens.accent_tint);

        assert_eq!(
            box_fill(&tokens, false, Pointer::Rest),
            button,
            "{theme:?}: off"
        );
        assert_eq!(
            box_fill(&tokens, true, Pointer::Rest),
            highlight,
            "{theme:?}: on is the solid highlight"
        );
        assert_eq!(
            box_fill(&tokens, false, Pointer::Hovered),
            darkly::lighter(button, 1.1),
            "{theme:?}: hover lighter(110)"
        );
        assert_eq!(
            box_fill(&tokens, true, Pointer::Hovered),
            darkly::lighter(highlight, 1.1),
            "{theme:?}: checked and hovered lighter(110)"
        );
        assert_eq!(
            box_fill(&tokens, false, Pointer::Pressed),
            darkly::lighter(button, 1.1),
            "{theme:?}: a press under the pointer is hover"
        );
    }
}

#[test]
fn a_check_mark_is_darkly_path_in_the_box() {
    let square = Rect::from_min_size(pos2(0.0, 0.0), vec2(16.0, 16.0));

    assert_eq!(
        partial(square, 1.0),
        vec![pos2(5.0, 8.5), pos2(7.0, 11.0), pos2(12.0, 5.0)]
    );
    assert_eq!(
        partial(square, 0.0).len(),
        1,
        "nothing drawn yet but the start"
    );
}
