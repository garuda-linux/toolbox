use toolbox_plan::PackageName;
use toolbox_system::{
    DiskSpace, Facts, HealthIssue, HealthLevel, PackageCount, PendingUpdate, ProbeError,
};

use super::{Figure, Standing, Stat, Tile, tiles, used_share};
use crate::home::Overview;
use crate::home::tests::{DAY, GB, now, quiet};

fn facts(free: u64) -> Facts {
    Facts {
        packages: Ok(PackageCount {
            installed: 1_462,
            foreign: None,
        }),
        kernel: Ok("6.17.7-zen1-1-zen".to_owned()),
        booted: Ok(now() - DAY),
        disk: Ok(DiskSpace {
            total: 500 * GB,
            free,
        }),
        package_cache: Ok(GB),
        orphans: Ok(Vec::new()),
        failed_units: Ok(Vec::new()),
    }
}

fn standings(tiles: &[Tile; 4]) -> [Standing; 4] {
    tiles.map(|tile| tile.standing)
}

#[test]
fn a_quiet_system_shows_four_fine_tiles_in_order() {
    let status = quiet();
    let read = facts(200 * GB);
    let overview = Overview::of(&status, now());
    let shown = tiles(Some(&overview), Some(&read));

    assert_eq!(shown.map(|tile| tile.stat), Stat::ALL);
    assert_eq!(standings(&shown), [Standing::Fine; 4]);
    assert_eq!(
        shown.map(|tile| tile.figure),
        [
            Figure::Updates(0),
            Figure::Free(200 * GB),
            Figure::Health(None),
            Figure::Days(2),
        ]
    );
}

#[test]
fn nothing_read_yet_is_unknown() {
    let shown = tiles(None, None);

    assert_eq!(standings(&shown), [Standing::Unknown; 4]);
    assert!(shown.iter().all(|tile| tile.figure == Figure::Unknown));
}

#[test]
fn updates_a_low_disk_health_findings_and_a_stale_system_need_attention() {
    let mut status = quiet();
    status.updates = Ok(vec![PendingUpdate {
        name: PackageName::new("firefox").unwrap(),
        from: toolbox_text::plain("144.0-1"),
        to: toolbox_text::plain("145.0-1"),
    }]);
    status.aur_updates = Ok(Vec::new());
    status.last_update = Ok(Some(now() - DAY * 30_i32));
    status.health = Ok(vec![HealthIssue {
        level: HealthLevel::High,
        message: toolbox_text::plain("old keyring"),
        fix_available: true,
    }]);

    let read = facts(9 * GB);
    let overview = Overview::of(&status, now());
    let shown = tiles(Some(&overview), Some(&read));

    assert_eq!(standings(&shown), [Standing::Attention; 4]);
    assert_eq!(shown[0].figure, Figure::Updates(1));
    assert_eq!(shown[2].figure, Figure::Health(Some(HealthLevel::High)));
    assert_eq!(shown[3].figure, Figure::Days(30));
}

#[test]
fn a_check_that_failed_leaves_its_tile_unknown() {
    let mut status = quiet();
    status.updates = Err(ProbeError::fake("checkupdates: no network"));
    status.health = Err(ProbeError::Unavailable);
    status.last_update = Ok(None);

    let mut read = facts(200 * GB);
    read.disk = Err(ProbeError::fake("statvfs: denied"));
    let overview = Overview::of(&status, now());

    assert_eq!(
        standings(&tiles(Some(&overview), Some(&read))),
        [Standing::Unknown; 4]
    );
}

#[test]
fn only_the_disk_tile_carries_the_share_in_use() {
    let status = quiet();
    let read = facts(200 * GB);
    let overview = Overview::of(&status, now());
    let shown = tiles(Some(&overview), Some(&read));

    assert_eq!(shown.map(|tile| tile.used), [None, Some(0.6), None, None]);
    assert_eq!(tiles(None, None)[1].used, None, "not read yet");
}

#[test]
fn the_share_in_use_is_the_room_not_free_of_the_whole() {
    assert_eq!(used_share(100 * GB, 400 * GB), Some(0.75));
    assert_eq!(used_share(0, 400 * GB), Some(1.0));
    assert_eq!(used_share(400 * GB, 400 * GB), Some(0.0));
}

#[test]
fn a_disk_without_size_has_no_share_and_more_free_than_whole_none_in_use() {
    assert_eq!(used_share(0, 0), None);
    assert_eq!(used_share(5 * GB, 0), None);
    assert_eq!(used_share(600 * GB, 500 * GB), Some(0.0));
}
