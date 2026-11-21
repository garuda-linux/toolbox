use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Color32, Rect, Theme};
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::{filled_rects, top_lights};
use toolbox_gui_kit::{Loadable, tokens};
use toolbox_system::DiskSpace;

use super::{GB, Shown, facts, page, status};
use crate::testing::PageFixture;

fn with_free(free: u64) -> Shown {
    let mut read = facts();
    read.disk = Ok(DiskSpace {
        total: 500 * GB,
        free,
    });

    page(PageFixture {
        status: Some(Loadable::Loaded(Arc::new(status()))),
        facts: Some(Loadable::Loaded(Arc::new(read))),
        ..PageFixture::default()
    })
}

fn disk(harness: &Shown) -> Rect {
    harness.get_by_label_contains("Disk: ").rect()
}

/// The bar's parts in `fill`: as tall as a meter, so the status mark's
/// glyph in the same colour is left out.
fn inside(harness: &Shown, fill: Color32) -> Vec<Rect> {
    let tile = disk(harness);
    let height = tokens::current(&harness.ctx).sizes.meter_height;
    filled_rects(harness, fill)
        .into_iter()
        .filter(|rect| tile.contains_rect(*rect) && (rect.height() - height).abs() < 0.01)
        .collect()
}

#[test]
fn the_disk_tile_fills_its_bar_with_the_share_in_use() {
    let harness = with_free(200 * GB);
    let tokens = tokens::current(&harness.ctx);

    let (track, used) = match (
        inside(&harness, tokens.field).as_slice(),
        inside(&harness, tokens.accent).as_slice(),
    ) {
        ([track], [used]) => (*track, *used),
        (tracks, used) => panic!("one track and one fill: {tracks:?} {used:?}"),
    };

    assert!(
        (used.width() / track.width() - 0.6).abs() < 0.01,
        "{used:?} of {track:?}"
    );
    assert!(track.left() > disk(&harness).left(), "inside the padding");
}

#[test]
fn a_disk_running_low_fills_its_bar_in_the_warning_tone() {
    let harness = with_free(9 * GB);
    let tokens = tokens::current(&harness.ctx);

    assert_eq!(inside(&harness, tokens.warning).len(), 1);
    assert_eq!(inside(&harness, tokens.accent).len(), 0);
}

#[test]
fn every_tile_keeps_the_room_of_the_bar() {
    let harness = with_free(200 * GB);
    let others = [
        "Updates: Up to date",
        "Health: No problems",
        "Last update: 16 days ago",
    ];

    let height = disk(&harness).height();
    for name in others {
        let tile = harness.get_by_role_and_label(Role::Link, name).rect();
        assert!((tile.height() - height).abs() < 0.5, "{name}: {tile:?}");
    }
}

#[test]
fn a_dark_tile_has_the_light_along_its_top() {
    let mut harness = with_free(200 * GB);
    harness.ctx.set_theme(Theme::Dark);
    harness.run();

    let tile = disk(&harness);
    let lit = top_lights(&harness)
        .into_iter()
        .map(egui::epaint::Mesh::calc_bounds)
        .filter(|light| {
            (light.top() - tile.top()).abs() <= 1.5 && tile.x_range().contains(light.center().x)
        })
        .count();

    assert_eq!(lit, 1);
}
