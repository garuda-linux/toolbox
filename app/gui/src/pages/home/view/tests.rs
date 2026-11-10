use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use toolbox_plan::{PackageName, UnitName};
use toolbox_system::{
    DiskSpace, Facts, HealthIssue, HealthLevel, PackageCount, PendingUpdate, ProbeError, Status,
};

use super::{Check, Overview, Pending};

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

fn update(name: &str, from: &str, to: &str) -> PendingUpdate {
    PendingUpdate {
        name: PackageName::new(name).unwrap(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn issue(level: HealthLevel, message: &str, fix_available: bool) -> HealthIssue {
    HealthIssue {
        level,
        message: message.to_owned(),
        fix_available,
    }
}

#[test]
fn a_quiet_system_needs_no_attention() {
    let status = quiet();
    let overview = Overview::of(&status, now());

    assert_eq!(overview.days, Some(2));
    assert!(!overview.stale);
    assert!(!overview.attention(false));
    assert!(overview.attention(true), "a pending restart");
}

#[test]
fn updates_are_listed_per_source_repositories_first() {
    let mut status = quiet();
    status.updates = Ok(vec![update("linux", "6.1-1", "6.2-1")]);
    status.aur_updates = Ok(vec![
        update("paru-bin", "2.0-1", "2.1-1"),
        update("yay-bin", "12.0-1", "12.1-1"),
    ]);

    let overview = Overview::of(&status, now());

    assert_eq!((overview.updates, overview.aur_updates), (1, 2));
    assert!(overview.attention(false));
    assert_eq!(
        overview.pending.first(),
        Some(&Pending {
            name: "linux",
            from: "6.1-1",
            to: "6.2-1",
            aur: false,
        })
    );
    assert_eq!(overview.pending.iter().filter(|row| row.aur).count(), 2);
}

#[test]
fn only_a_failed_check_is_listed_with_its_reason() {
    let mut status = quiet();
    status.updates = Err(ProbeError::Failed("no database".to_owned()));
    status.health = Err(ProbeError::Failed("no garuda-health".to_owned()));

    let overview = Overview::of(&status, now());

    assert_eq!(overview.updates, 0);
    assert_eq!(
        overview.failed,
        [
            (Check::Updates, "no database".into()),
            (Check::Health, "no garuda-health".into()),
        ]
    );
    assert_eq!(Overview::of(&quiet(), now()).failed, []);
}

#[test]
fn merges_are_listed_as_paths() {
    let mut status = quiet();
    status.merges = Ok(vec![PathBuf::from("/etc/pacman.conf.pacnew")]);

    let overview = Overview::of(&status, now());

    assert_eq!(overview.merges, [PathBuf::from("/etc/pacman.conf.pacnew")]);
    assert!(overview.attention(false));
}

#[test]
fn the_system_is_stale_after_two_weeks() {
    let mut status = quiet();
    status.last_update = Ok(Some(now() - DAY * 15));

    let overview = Overview::of(&status, now());

    assert_eq!((overview.days, overview.stale), (Some(15), true));
    assert!(overview.attention(false));

    status.last_update = Ok(None);
    let never = Overview::of(&status, now());
    assert_eq!((never.days, never.stale), (None, false));
}

#[test]
fn health_names_its_worst_level_above_info() {
    let mut status = quiet();
    status.health = Ok(vec![issue(HealthLevel::Info, "fine", false)]);

    let calm = Overview::of(&status, now());
    assert_eq!(calm.health, None);
    assert!(!calm.attention(false));

    status.health = Ok(vec![
        issue(HealthLevel::Info, "fine", false),
        issue(HealthLevel::Low, "old keyring", false),
        issue(HealthLevel::Critical, "broken boot entry", true),
    ]);

    let warned = Overview::of(&status, now());
    assert_eq!(warned.health, Some(HealthLevel::Critical));
    assert_eq!(warned.health_messages, ["old keyring", "broken boot entry"]);
    assert!(warned.health_fixable);
}

fn facts() -> Facts {
    Facts {
        packages: Ok(PackageCount {
            installed: 1_462,
            foreign: Some(12),
        }),
        kernel: Ok("6.17.7-zen1-1-zen".to_owned()),
        booted: Ok(now() - DAY * 4),
        disk: Ok(DiskSpace {
            total: 500 * GB,
            free: 200 * GB,
        }),
        package_cache: Ok(GB),
        orphans: Ok(Vec::new()),
        failed_units: Ok(Vec::new()),
    }
}

#[test]
fn quiet_facts_need_no_attention() {
    let status = quiet();
    let facts = facts();
    let overview = Overview::of(&status, now()).with_facts(&facts);

    assert!(!overview.machine.disk_low);
    assert!(!overview.machine.cleanup);
    assert_eq!(overview.machine.failed_units.len(), 0, "no failed unit");
    assert!(!overview.attention(false));
}

#[test]
fn a_low_disk_a_failed_unit_and_orphans_need_attention() {
    let status = quiet();
    let mut facts = facts();
    facts.disk = Ok(DiskSpace {
        total: 500 * GB,
        free: 9 * GB,
    });
    facts.failed_units = Ok(vec![UnitName::new("smb.service").unwrap()]);
    facts.orphans = Ok(vec![PackageName::new("old-lib").unwrap()]);
    let overview = Overview::of(&status, now()).with_facts(&facts);

    assert!(overview.machine.disk_low);
    assert_eq!(overview.machine.failed_units.len(), 1);
    assert!(overview.machine.cleanup);
    assert_eq!(overview.machine.orphans, 1);
    assert!(overview.attention(false));
}

#[test]
fn a_fact_that_failed_is_listed_with_its_reason() {
    let status = quiet();
    let mut facts = facts();
    facts.disk = Err(ProbeError::Failed("statvfs: denied".to_owned()));
    facts.failed_units = Err(ProbeError::Unavailable);
    facts.orphans = Err(ProbeError::Failed("no database".to_owned()));
    let overview = Overview::of(&status, now()).with_facts(&facts);

    assert!(overview.failed_check(Check::Disk));
    assert!(overview.failed_check(Check::Cleanup));
    assert!(
        !overview.failed_check(Check::Services),
        "not available is no failure"
    );
    assert!(!overview.machine.disk_low);
}

#[test]
fn without_facts_only_the_status_checks_ran() {
    let status = quiet();
    let overview = Overview::of(&status, now());

    assert!(!overview.machine.disk_low);
    assert_eq!(overview.ran, [Check::Updates, Check::Merges, Check::Health]);
}

#[test]
fn every_check_that_gave_an_answer_ran() {
    let status = quiet();
    let facts = facts();
    let overview = Overview::of(&status, now()).with_facts(&facts);

    // The AUR check is not available on the quiet system.
    assert_eq!(
        overview.ran,
        [
            Check::Updates,
            Check::Merges,
            Check::Disk,
            Check::Services,
            Check::Cleanup,
            Check::Health,
        ]
    );
}
