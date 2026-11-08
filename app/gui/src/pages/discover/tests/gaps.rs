//! The room above an app row's capitals against the room under its last
//! baseline, as every card row keeps them.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::{capitals_top, last_baseline_within};
use toolbox_gui_kit::tokens;

use super::{offered, searching_firefox};

/// Off by at most this, two gaps count as one.
const SAME: f32 = 1.0;

/// Firefox's description takes one line of the two a row may show, so
/// a row as tall as two lines would leave room under it.
#[test]
fn an_app_rows_capitals_have_as_much_room_above_as_its_last_baseline_below() {
    let mut harness = offered().harness(searching_firefox());
    harness.run();

    let row = harness
        .get_by_role_and_label(Role::ListItem, "Firefox")
        .rect();

    let capitals = capitals_top(&harness, "Firefox").expect("the title is painted");
    let (baseline, last) = last_baseline_within(&harness, row).expect("the text is painted");
    let above = capitals - row.top();
    let below = row.bottom() - baseline;
    let padding = tokens::Sizes::DEFAULT.row_padding.y;

    assert!(
        (above - below).abs() <= SAME,
        "the capitals start {above} under the row's top, the last baseline ends {below} above \
         its bottom"
    );
    assert!(
        (row.bottom() - last - padding).abs() <= SAME,
        "the last line ends {} above the row's bottom",
        row.bottom() - last
    );
}
