use egui::{Rect, pos2};

use super::{Fit, Place, place};

const GAP: f32 = 8.0;

fn window() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 700.0))
}

fn pill_at(top: f32) -> Rect {
    Rect::from_min_max(pos2(700.0, top), pos2(980.0, top + 40.0))
}

/// A popover of 120 points round a list of `list` points.
fn fit(list: f32) -> Fit {
    Fit {
        chrome: 120.0,
        list,
        most: 320.0,
    }
}

#[test]
fn it_opens_above_a_pill_at_the_bottom() {
    let placed = place(pill_at(640.0), window(), fit(200.0), GAP);

    assert_eq!(
        placed,
        Place {
            above: true,
            list: 320.0
        }
    );
}

#[test]
fn it_opens_below_a_pill_too_high_for_it_above() {
    let placed = place(pill_at(200.0), window(), fit(200.0), GAP);

    assert_eq!(
        placed,
        Place {
            above: false,
            list: 320.0
        }
    );
}

/// The list's own most counts, not a longer list that scrolls in it.
#[test]
fn a_long_list_counts_as_tall_as_it_shows() {
    let placed = place(pill_at(500.0), window(), fit(2000.0), GAP);

    assert!(placed.above, "{placed:?}");
}

/// No side has room: the roomier one, with the list shortened so the
/// popover keeps its gap to the window's edge.
#[test]
fn without_room_either_side_the_list_shortens_on_the_roomier() {
    let placed = place(pill_at(300.0), window(), fit(2000.0), GAP);

    assert_eq!(
        placed,
        Place {
            above: false,
            list: 700.0 - 340.0 - GAP - GAP - 120.0
        }
    );
}
