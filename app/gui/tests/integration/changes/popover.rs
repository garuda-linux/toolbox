//! Where the popover opens and which layer it lies on.

use egui_kittest::kittest::Queryable as _;

use super::{app_with, queued, review};
use crate::common::{WIDTH, installing, launch_in};

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
    let title = harness.get_by_label("Queued changes").rect();
    let least = sizes.popover_gap + f32::from(sizes.popover_padding.top);

    assert!(
        title.top() >= least - 0.5,
        "the title is at {}",
        title.top()
    );
}

/// egui raises an area in the first frame it shows, so a pill that comes
/// back while a dialog is open, as when the Console closes, would cover the
/// dialog's buttons on the dialogs' own layer. It lies under them instead.
#[test]
fn the_pill_and_its_popover_lie_under_every_dialog() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);

    let ours = [
        "toolbox_gui::changes::pill",
        "toolbox_gui::changes::popover",
    ]
    .map(egui::Id::unique);

    let layers: Vec<egui::LayerId> = harness.ctx.memory(|memory| {
        memory
            .layer_ids()
            .filter(|layer| ours.contains(&layer.id))
            .collect()
    });

    assert_eq!(layers.len(), 2, "{layers:?}");
    assert!(
        layers
            .iter()
            .all(|layer| layer.order < egui::Order::Foreground),
        "{layers:?}"
    );
}
