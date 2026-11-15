use std::time::{Duration, SystemTime};

use toolbox_plan::PackageName;
use toolbox_system::{
    DiskSpace, Facts, HealthIssue, HealthLevel, PackageCount, PendingUpdate, ProbeError, Status,
};

use super::{Figure, Standing, Stat, Tile, tiles};
use crate::home::Overview;

const DAY: Duration = Duration::from_hours(24);
const GB: u64 = 1_000_000_000;

fn now() -> SystemTime {
    SystemTime::UNIX_EPOCH + DAY * 20_000
}

fn quiet() -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Err(ProbeError::Unavailable),
        merges: Ok(Vec::new()),
        last_update: Ok(Some(now() - DAY * 2)),
        health: Ok(Vec::new()),
    }
}

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
        from: "144.0-1".to_owned(),
        to: "145.0-1".to_owned(),
    }]);
    status.aur_updates = Ok(Vec::new());
    status.last_update = Ok(Some(now() - DAY * 30));
    status.health = Ok(vec![HealthIssue {
        level: HealthLevel::High,
        message: "old keyring".to_owned(),
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
