use std::time::{Duration, SystemTime};

use toolbox_cli::output::{Paint, Style};
use toolbox_plan::{PackageName, UnitName};
use toolbox_system::{DiskSpace, Facts, PackageCount, ProbeError, Status};

use super::super::{brief, needs_attention, text, view};

const DAY: Duration = Duration::from_hours(24);
const GB: u64 = 1_000_000_000;

fn quiet() -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(Vec::new()),
        merges: Ok(Vec::new()),
        last_update: Ok(None),
        health: Ok(Vec::new()),
    }
}

fn facts(now: SystemTime) -> Facts {
    Facts {
        packages: Ok(PackageCount {
            installed: 1_462,
            foreign: Some(12),
        }),
        kernel: Ok("6.17.7-zen1-1-zen".to_owned()),
        booted: Ok(now - DAY * 4),
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
fn quiet_facts_are_all_good() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = view(&quiet(), &Ok(false), now).with_facts(&facts(now), now);

    assert_eq!(brief(&shown), "all good");
    assert!(!needs_attention(&shown));
}

#[test]
fn a_low_disk_a_failed_service_and_orphans_need_attention() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = facts(now);
    found.disk = Ok(DiskSpace {
        total: 500 * GB,
        free: 9 * GB,
    });
    found.failed_units = Ok(vec![UnitName::new("smb.service").unwrap()]);
    found.orphans = Ok(vec![PackageName::new("old-lib").unwrap()]);
    let shown = view(&quiet(), &Ok(false), now).with_facts(&found, now);

    assert_eq!(
        brief(&shown),
        "disk space low, 1 failed service, clean-up due"
    );
    assert!(needs_attention(&shown));
}

#[test]
fn the_text_lists_the_facts_after_the_update_check() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = view(&quiet(), &Ok(false), now).with_facts(&facts(now), now);
    let printed = text(&shown, Paint::new(Style::Plain), false);

    assert!(printed.contains("1462 (12 foreign)"), "{printed}");
    assert!(printed.contains("6.17.7-zen1-1-zen"), "{printed}");
    assert!(printed.contains("4 days"), "{printed}");
    assert!(printed.contains("186.3 GiB free of 465.7 GiB"), "{printed}");
    assert!(printed.contains("953.7 MiB"), "{printed}");
}

#[test]
fn the_json_names_each_fact_with_its_state() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = facts(now);
    found.orphans = Err(ProbeError::Failed("no database".to_owned()));
    let shown = view(&quiet(), &Ok(false), now).with_facts(&found, now);
    let json = toolbox_cli::output::json(&shown);

    assert!(json.contains("\"kernel\""), "{json}");
    assert!(json.contains("\"uptime_hours\""), "{json}");
    assert!(json.contains("\"failed_services\""), "{json}");
    assert!(json.contains("\"state\": \"failed\""), "{json}");
}

#[test]
fn a_view_without_facts_keeps_them_out_of_the_json() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let json = toolbox_cli::output::json(&view(&quiet(), &Ok(false), now));

    assert!(!json.contains("\"kernel\""), "{json}");
}
