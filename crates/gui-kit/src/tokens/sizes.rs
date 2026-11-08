//! The room between things, after Rerun's `DesignTokens`: pages ask for
//! a gap by its role, so one change here moves every page alike.

use egui::{Margin, Vec2, vec2};

/// Gaps, paddings and row heights in points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    /// Room around a page's content.
    pub page_padding: Margin,
    /// Inside a card row.
    pub row_padding: Vec2,
    /// Between the large blocks of a page, such as Home's greeting and
    /// its status.
    pub block_gap: f32,
    /// Below a page's header and below its filters.
    pub header_gap: f32,
    /// Below a page's chips, above its first section.
    pub filters_gap: f32,
    /// Below a section, above the next one's heading.
    pub section_gap: f32,
    /// Below a heading, above what it heads.
    pub heading_gap: f32,
    /// Between cards, banners and tiles of one column.
    pub card_gap: f32,
    /// Above a dialog's buttons.
    pub dialog_gap: f32,
    /// Between controls side by side.
    pub control_gap: f32,
    /// A caption beside or below its title.
    pub caption_gap: f32,
    /// Between the parts of one item: a title and its description, a
    /// label and its button.
    pub tight_gap: f32,
    /// A label's room from the edge of the pill or panel it sits in.
    pub inset: f32,
    /// A small picture or icon and the text beside it.
    pub text_to_icon: f32,
    /// The tinted square behind a row's icon, and behind a section's or a
    /// one-line row's.
    pub icon_tile: f32,
    pub icon_tile_small: f32,
    /// A row's 32 px picture and the title beside it, on every page alike.
    pub picture_gap: f32,
    /// Home's logo and the greeting beside it.
    pub logo_gap: f32,
    /// From the page's bottom corners to what floats there: the reading
    /// note, and the Changes pill's right edge.
    pub corner_inset: Vec2,
    /// The band below the page that holds the Changes pill while changes
    /// wait, so the pill covers nothing.
    pub changes_band: f32,
    /// Between a pill and the popover it opens.
    pub popover_gap: f32,
    /// Between the sidebar's rows.
    pub nav_gap: f32,
    /// Between the sidebar's groups when their headings are hidden.
    pub nav_group_gap: f32,
    /// A sidebar entry.
    pub nav_row_height: f32,
    /// The gaps while the entries at normal size would not fit the
    /// sidebar's height. The entries keep their height and the list scrolls.
    pub nav_gap_compact: f32,
    pub nav_group_gap_compact: f32,
    /// The least room between a collapsed sidebar's group divider and the
    /// entries beside it, packed or not.
    pub nav_divider_room: f32,
    /// Inside the sidebar.
    pub sidebar_padding: Margin,
    /// Inside the status bar at the window's foot.
    pub status_bar_padding: Margin,
    /// Inside the Console sheet.
    pub sheet_padding: Margin,
    /// Around the Console's list of steps, inside its card.
    pub steps_padding: Margin,
    /// Between the terminal's frame and its text.
    pub terminal_padding: Margin,
    /// Inside the Changes popover.
    pub popover_padding: Margin,
    /// Inside the Changes pill, around its buttons.
    pub pill_padding: Margin,
    /// A dialog for a short question or a short list, such as the palette.
    pub dialog_small: f32,
    /// A dialog with a form or a few paragraphs.
    pub dialog_medium: f32,
    /// A dialog that shows the lines of a file.
    pub dialog_wide: f32,
    /// A page's search field, the same on every page.
    pub search_width: f32,
    /// A sort dropdown in a page's toolbar.
    pub sort_width: f32,
    /// The control column of a setting row: a dropdown or a field beside
    /// its title. Narrow for short choices, wide for long ones.
    pub setting_control_narrow: f32,
    pub setting_control_wide: f32,
    /// The buttons at the right of a list row. Narrow for one or two short
    /// ones, wide for more.
    pub row_actions_narrow: f32,
    pub row_actions_wide: f32,
    /// Inside a card that holds no rows, such as About's facts.
    pub card_padding: Margin,
    /// Around the rows of a card of links, so their hover fill keeps off
    /// the card's edge.
    pub list_padding: Margin,
}

impl Sizes {
    pub const DEFAULT: Self = Self {
        page_padding: Margin {
            left: 40,
            right: 40,
            top: 32,
            bottom: 32,
        },
        row_padding: vec2(20.0, 12.0),
        block_gap: 32.0,
        header_gap: 20.0,
        filters_gap: 24.0,
        section_gap: 42.0,
        heading_gap: 10.0,
        card_gap: 12.0,
        dialog_gap: 16.0,
        control_gap: 8.0,
        caption_gap: 6.0,
        tight_gap: 4.0,
        inset: 6.0,
        text_to_icon: 6.0,
        icon_tile: 36.0,
        icon_tile_small: 28.0,
        picture_gap: 10.0,
        logo_gap: 18.0,
        corner_inset: vec2(28.0, 24.0),
        changes_band: 72.0,
        popover_gap: 10.0,
        nav_gap: 2.0,
        nav_group_gap: 6.0,
        nav_row_height: 32.0,
        nav_gap_compact: 1.0,
        nav_group_gap_compact: 2.0,
        nav_divider_room: 6.0,
        sidebar_padding: Margin {
            left: 12,
            right: 12,
            top: 14,
            bottom: 12,
        },
        status_bar_padding: Margin::symmetric(16, 6),
        sheet_padding: Margin {
            left: 24,
            right: 24,
            top: 10,
            bottom: 18,
        },
        steps_padding: Margin::symmetric(14, 8),
        terminal_padding: Margin::same(12),
        popover_padding: Margin::same(16),
        pill_padding: Margin::same(6),
        dialog_small: 460.0,
        dialog_medium: 560.0,
        dialog_wide: 720.0,
        search_width: 240.0,
        sort_width: 200.0,
        setting_control_narrow: 260.0,
        setting_control_wide: 340.0,
        row_actions_narrow: 260.0,
        row_actions_wide: 320.0,
        card_padding: Margin::symmetric(20, 14),
        list_padding: Margin::same(6),
    };
}

impl Default for Sizes {
    fn default() -> Self {
        Self::DEFAULT
    }
}
