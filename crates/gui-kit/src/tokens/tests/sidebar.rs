//! The sidebar takes the header's look: the header's scrim laid on the
//! plain page, opaque, with every entry, icon and label readable on it.

use super::super::contrast;
use super::every_tokens;

#[test]
fn the_sidebar_is_the_header_band_over_the_plain_page() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(
            tokens.sidebar,
            tokens.window.blend(tokens.header_scrim),
            "{theme:?}"
        );
    }
}

#[test]
fn the_sidebar_stays_opaque() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(tokens.sidebar.a(), 255, "{theme:?}");
    }
}

#[test]
fn the_console_sheet_stands_apart_from_its_terminal() {
    for (theme, tokens) in every_tokens() {
        let apart = contrast(tokens.pane, tokens.terminal);
        assert!(
            apart >= 1.05,
            "{theme:?}: pane and terminal at {apart:.2}:1"
        );
    }
}

#[test]
fn every_text_on_the_sidebar_and_its_status_bar_reads_at_four_and_a_half() {
    for (theme, tokens) in every_tokens() {
        for (name, ink) in [
            ("text", tokens.text),
            ("entries and group labels", tokens.sidebar_text),
            ("secondary", tokens.secondary),
            ("success", tokens.success),
            ("warning", tokens.warning),
            ("error", tokens.error),
        ] {
            let ratio = contrast(ink, tokens.sidebar);
            assert!(ratio >= 4.5, "{theme:?} {name}: {ratio:.2}");
        }
    }
}

#[test]
fn icons_and_dim_text_show_on_the_sidebar_at_three() {
    for (theme, tokens) in every_tokens() {
        for (name, ink) in [("icons", tokens.sidebar_text), ("dim", tokens.dim)] {
            let ratio = contrast(ink, tokens.sidebar);
            assert!(ratio >= 3.0, "{theme:?} {name}: {ratio:.2}");
        }
    }
}

#[test]
fn the_selected_entry_s_text_reads_at_four_and_a_half_to_one() {
    for (theme, tokens) in every_tokens() {
        assert_eq!(tokens.on_tint, tokens.highlighted_text, "{theme:?}");
        let ratio = contrast(tokens.on_tint, tokens.accent_tint);
        assert!(ratio >= 4.5, "{theme:?}: {ratio:.2}:1");
    }
}
