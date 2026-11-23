//! Where the popover opens and which layer it lies on.

use egui_kittest::kittest::Queryable as _;

use super::{app_with, bar_rect, footer_id, queued, review};
use crate::common::{WIDTH, installing, launch_in, pane, pointer_button};

/// A queue too long for the room above the pill keeps a gap to the
/// window's top and scrolls a shorter list.
#[test]
fn a_long_queue_in_a_short_window_keeps_off_the_top_edge() {
    let names: Vec<String> = (0_u8..30).map(|n| format!("package{n}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let short = toolbox_gui_kit::testing::harness(WIDTH, 400.0);
    let mut harness = launch_in(short, queued(installing(&names)));
    review(&mut harness);

    let sizes = toolbox_gui_kit::tokens::current(&harness.ctx).sizes;
    let title = harness.get_by_label("Queued Changes").rect();
    let least = sizes.popover_gap + f32::from(sizes.popover_padding.top);

    assert!(
        title.top() >= least - 0.5,
        "the title is at {}",
        title.top()
    );
}

/// egui raises an area in the first frame it shows, so a footer that comes
/// back while a dialog is open, as when the Console closes, would cover the
/// dialog's buttons on the dialogs' own layer. It lies under them instead.
#[test]
fn the_footer_lies_under_every_dialog() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);

    let footer = footer_id();
    let layers: Vec<egui::LayerId> = harness.ctx.memory(|memory| {
        memory
            .layer_ids()
            .filter(|layer| layer.id == footer)
            .collect()
    });

    assert_eq!(layers.len(), 1, "{layers:?}");
    assert!(
        layers
            .iter()
            .all(|layer| layer.order < egui::Order::Foreground),
        "{layers:?}"
    );
}

/// egui keeps every popup on the dialogs' layer, so the popover steps
/// aside while the question asks and comes back after it.
#[test]
fn the_popover_steps_aside_for_the_question_and_comes_back() {
    let mut harness = app_with(&["firefox", "vlc"]);
    review(&mut harness);

    harness.get_by_label("Remove All…").click();
    harness.run();
    assert!(pane(&harness, "Changes to apply").is_none());

    harness.get_by_label("Keep Them").click();
    harness.run();
    assert!(pane(&harness, "Changes to apply").is_some());
}

/// Review sits in the footer, so the list rises from it and never over it.
#[test]
fn the_popover_opens_upward_from_the_footer() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);

    let bar = bar_rect(&harness);
    let list = pane(&harness, "Changes to apply").expect("the popover");

    assert!(
        list.bottom() <= bar.top(),
        "the list {list:?} lies over the footer {bar:?}"
    );
}

#[test]
fn a_click_beside_the_popover_closes_it() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);
    let page = pane(&harness, "Page").expect("the page");
    let beside = page.left_top() + egui::vec2(12.0, 12.0);

    // A press and its release in the next frame, quick enough for a click.
    harness.event(egui::Event::PointerMoved(beside));
    harness.step();
    harness.event(pointer_button(beside, true));
    harness.step();
    harness.event(pointer_button(beside, false));
    harness.run();

    assert!(pane(&harness, "Changes to apply").is_none());
    assert!(harness.query_by_label("Review").is_some());
}
