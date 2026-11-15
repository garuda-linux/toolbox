//! The app's own controls, painted by hand as spotifast's are.
//!
//! Each takes a rect from `allocate_exact_size`, its accessible name and
//! role, the shapes from the tokens, and one focus ring. Pages build from
//! these and never pick colours or sizes themselves.

mod action_row;
mod banner;
pub(crate) mod buttons;
mod card;
mod check;
mod check_row;
mod chip;
mod choice;
mod dim;
mod empty;
mod field;
mod focus;
mod heading;
mod highlight;
mod icon_tile;
mod layout;
mod letter_tile;
mod link_row;
mod list;
mod loading;
mod mark_word;
mod menu;
mod metric;
mod modal;
mod nav;
mod number;
mod page;
mod painted_row;
mod press;
mod property;
pub(crate) mod removable;
mod roving;
mod row_menu;
mod search;
mod secret;
mod secret_field;
mod segmented;
mod select;
mod skeleton;
mod spin;
mod status_tile;
mod switch;
#[cfg(test)]
mod tests;
mod text;
mod text_box;
mod text_link;
mod tile;

pub use action_row::ActionRow;
pub use banner::banner;
pub use buttons::{
    ButtonKind, icon_button, icon_button_width, named_pill_button, pill_button,
    pill_button_at_least, pill_width, refresh_button, right_buttons, sidebar_button,
};
pub use card::{card, card_row, card_row_with, padded_card};
pub use check_row::{CHECK_ROW_HEIGHT, CheckRowTexts, check_row, check_row_with_line};
pub use chip::{
    Tone, chip, chips, counted_chips, link_tag, named_chips, queued_tag, status_dot, tag,
    toggle_chips,
};
pub use choice::Choice;
pub use dim::{dim_opacity, dim_picture, dimmed};
pub use empty::{empty_state, empty_state_action};
pub use field::{forget_text, text_field, text_field_with_unit};
pub use focus::{edge_ring, focus_ring, install as install_focus};
use heading::mark_heading;
pub use heading::{
    SectionHeading, minor_heading, page_header, page_toolbar, section_heading, section_intro,
};
pub use highlight::{highlighted, matches};
pub use icon_tile::{TileSize, icon_tile, page_tile, paint_icon_tile};
pub use layout::{
    MIN_TEXT_WIDTH, split_row, title_capitals_top, title_line, title_line_height, trailing,
};
pub use letter_tile::{hue_of, initial, paint_letter_tile};
pub use link_row::link_row;
use list::text_line_height;
pub use list::{ListItem, RowHeights, RowList, RowState, Rows, row_height, text_block_height};
pub use loading::{loading_indicator, loading_label};
pub use menu::{menu_action, menu_button, menu_heading, menu_item};
pub use metric::{MetricKind, MetricText, metric};
pub use modal::{
    Asked, Confirmation, DialogWidth, ModalOutcome, ModalWrapper, Pending, Question,
    confirm_buttons, confirm_buttons_when, confirm_buttons_why, question,
};
use nav::mark_selected;
pub use nav::{
    EdgeFade, NAV_INSET, NavLook, group_label, group_label_height, nav_divider, nav_divider_height,
    nav_row,
};
pub use number::{number_field, number_field_with_unit};
pub use page::{BodyShape, PageParts, cover_page_bottom, page_layout_shaped, start_page_entry};
pub use property::{PropertyList, PropertyTexts, property_list};
pub use removable::{locked_tag, removable_tag};
pub use row_menu::{row_context_menu, row_menu};
pub use search::{SearchTexts, request_search_focus, search_field};
pub use secret::SecretText;
pub use secret_field::{SecretTexts, secret_field};
pub use segmented::{segmented, segmented_width};
pub use select::Select;
pub use skeleton::{SkeletonRows, after_skeleton, hold_content, skeleton_list, skeleton_rows};
pub use spin::{SpinTexts, spin_field};
pub use status_tile::{paint_status_tile, status_tile, status_word};
pub use switch::{switch, tristate_switch};
pub use text::{line, mono_small, rich};
pub use text_box::{text_box, text_box_at, text_box_filling};
pub use text_link::{counted_disclosure, disclosure, text_button, text_link};
pub use tile::link_tile;
