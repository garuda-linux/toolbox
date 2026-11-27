//! The room between things, after Rerun's `DesignTokens`: pages ask for
//! a gap by its role, so one change here moves every page alike.

use egui::{Margin, Vec2, vec2};

use crate::theme::{LARGE_SPACING, SMALL_SPACING};

/// `FormCard`'s inner padding: large plus small spacing on every side.
const FORM_PADDING: f32 = LARGE_SPACING + SMALL_SPACING;

/// The footer margin of `KCMUtils` (`margins`, Breeze's `Layout_ChildMarginWidth`)
/// on every side of the footer's buttons.
const FOOTER_MARGIN: f32 = 6.0;

/// The margins above and below the buttons together.
const FOOTER_MARGINS: f32 = FOOTER_MARGIN + FOOTER_MARGIN;

/// The Changes footer above its button: a 1 px separator and the margins.
const CHANGES_CHROME: f32 = 1.0 + FOOTER_MARGINS;

/// Gaps, paddings and row heights in points. Spacing follows Kirigami's
/// units (`FormCard` and `ScrollablePage`), not a scale of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sizes {
    /// Room around a page's content.
    pub page_padding: Margin,
    /// Inside a card row.
    pub row_padding: Vec2,
    /// Between the large blocks of a page, such as Home's greeting and
    /// its status.
    pub block_gap: f32,
    /// The page's toolbar row, with its title and tabs, and the separator
    /// under it. Kirigami's global toolbar (`preferredHeight`, 40 + 6): a
    /// 40 px item of Breeze's toolbar plus its 6 px `ToolBar_ItemMargin`.
    pub toolbar_height: f32,
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
    /// A row's or a page title's text and the controls beside it, and a
    /// property's label and its value.
    pub text_to_control: f32,
    pub meter_height: f32,
    /// A row's 32 px picture and the title beside it, on every page alike.
    pub picture_gap: f32,
    /// Home's logo and the greeting beside it.
    pub logo_gap: f32,
    /// From the page's bottom corners to what floats there: the reading
    /// note.
    pub corner_inset: Vec2,
    /// The footer under the page while changes wait, without its button:
    /// the separator and the margins `KCMUtils` puts around it. The band
    /// adds the push button height.
    pub changes_chrome: f32,
    /// Between a pill and the popover it opens.
    pub popover_gap: f32,
    /// Between the sidebar's groups when their headings are hidden.
    pub nav_group_gap: f32,
    /// A sidebar entry, the pitch of KDE's sidebar.
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
    /// The Changes popover, and its list's height before it scrolls.
    pub popover_width: f32,
    pub changes_list_height: f32,
    /// The room a queued change keeps at its right for its remove button.
    pub remove_width: f32,
    /// The cell of a queued change's dot: its width and the first line's
    /// height.
    pub dot_cell: Vec2,
    /// The Console's list of steps before it scrolls, and one step.
    pub steps_height: f32,
    pub step_row_height: f32,
    /// Below this height the grip lets go of the Console sheet and it
    /// closes.
    pub sheet_least: f32,
    /// The page room the Console sheet always leaves above itself.
    pub sheet_room_above: f32,
    /// The Console sheet's grip row, and the bar drawn in it.
    pub grip_height: f32,
    pub grip_bar: Vec2,
    /// The palette's list before it scrolls.
    pub palette_list_height: f32,
    /// The collapsed sidebar: one icon row and the panel margins.
    pub sidebar_collapsed: f32,
    /// A dialog for a short question or a short list, such as the palette.
    pub dialog_small: f32,
    /// A dialog with a form or a few paragraphs.
    pub dialog_medium: f32,
    /// A dialog that shows the lines of a file.
    pub dialog_wide: f32,
    /// A page's search field, the same on every page.
    pub search_width: f32,
    /// The control column of a setting row: a dropdown or a field beside
    /// its title. Narrow for short choices, wide for long ones.
    pub setting_control_narrow: f32,
    pub setting_control_wide: f32,
    /// The buttons at the right of a list row, for one or two short ones.
    pub row_actions_narrow: f32,
    /// Inside a card that holds no rows, such as About's facts.
    pub card_padding: Margin,
    /// Around the rows of a card of links, so their hover fill keeps off
    /// the card's edge.
    pub list_padding: Margin,
}

impl Sizes {
    pub const DEFAULT: Self = Self {
        // ScrollablePage's padding is one grid unit on every side.
        page_padding: Margin::same(18),
        row_padding: vec2(FORM_PADDING, FORM_PADDING),
        block_gap: 32.0,
        toolbar_height: 46.0,
        // FormHeader: small spacing below the header and below a section's
        // heading, large plus small above a section.
        header_gap: SMALL_SPACING,
        filters_gap: 24.0,
        section_gap: LARGE_SPACING + SMALL_SPACING,
        heading_gap: SMALL_SPACING,
        card_gap: LARGE_SPACING,
        dialog_gap: 16.0,
        control_gap: LARGE_SPACING,
        caption_gap: SMALL_SPACING,
        tight_gap: SMALL_SPACING,
        inset: 6.0,
        text_to_icon: LARGE_SPACING,
        text_to_control: LARGE_SPACING,
        meter_height: 4.0,
        picture_gap: 10.0,
        logo_gap: 18.0,
        corner_inset: vec2(28.0, 24.0),
        changes_chrome: CHANGES_CHROME,
        popover_gap: 10.0,
        nav_group_gap: 6.0,
        nav_row_height: 36.0,
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
        popover_padding: Margin::same(12),
        popover_width: 380.0,
        changes_list_height: 320.0,
        remove_width: 32.0,
        dot_cell: vec2(8.0, 18.0),
        steps_height: 120.0,
        step_row_height: 20.0,
        sheet_least: 120.0,
        sheet_room_above: 48.0,
        grip_height: 10.0,
        grip_bar: vec2(40.0, 4.0),
        palette_list_height: 380.0,
        sidebar_collapsed: 60.0,
        dialog_small: 460.0,
        dialog_medium: 560.0,
        dialog_wide: 720.0,
        search_width: 240.0,
        setting_control_narrow: 260.0,
        setting_control_wide: 340.0,
        row_actions_narrow: 260.0,
        card_padding: Margin::symmetric(12, 12),
        list_padding: Margin::same(4),
    };
}

impl Default for Sizes {
    fn default() -> Self {
        Self::DEFAULT
    }
}
