use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use toolbox_cli::output::{Paint, Style};
use toolbox_plan::PackageName;
use toolbox_system::{HealthIssue, HealthLevel, PendingUpdate, ProbeError, Status};

use super::{Part, brief, needs_attention, text, view};

const DAY: Duration = Duration::from_hours(24);

fn update(name: &str) -> PendingUpdate {
    PendingUpdate {
        name: PackageName::new(name).unwrap(),
        from: "1".to_owned(),
        to: "2".to_owned(),
    }
}

fn status(now: SystemTime) -> Status {
    Status {
        updates: Ok(vec![update("firefox"), update("linux")]),
        aur_updates: Err(ProbeError::Unavailable),
        merges: Ok(vec![PathBuf::from("/etc/pacman.conf.pacnew")]),
        last_update: Ok(Some(now - DAY * 16)),
        health: Ok(vec![HealthIssue {
            level: HealthLevel::Low,
            message: "The keyring is older than three months.".to_owned(),
            fix_available: true,
        }]),
    }
}

#[test]
fn each_part_keeps_its_own_answer() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = view(&status(now), &Ok(true), now);

    assert!(matches!(&shown.updates, Part::Ok { value } if value.len() == 2));
    assert!(matches!(shown.aur_updates, Part::Unavailable));
    assert!(matches!(
        shown.last_update_days,
        Part::Ok { value: Some(16) }
    ));
    assert!(matches!(shown.restart_needed, Part::Ok { value: true }));
}

#[test]
fn the_text_names_what_needs_attention() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = text(
        &view(&status(now), &Ok(false), now),
        Paint::new(Style::Plain),
        false,
    );

    assert!(
        shown.contains("Updates") && shown.contains("2 packages"),
        "{shown}"
    );
    assert!(shown.contains("not available"), "{shown}");
    assert!(shown.contains("16 days ago"), "{shown}");
    assert!(
        shown.contains("The keyring is older than three months."),
        "{shown}"
    );
    assert!(
        shown.contains("Restart") && shown.contains("not needed"),
        "{shown}"
    );
}

#[test]
fn the_json_tells_every_part_by_its_state() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let json = toolbox_cli::output::json(&view(&status(now), &Ok(false), now));

    assert!(json.contains("\"state\": \"unavailable\""), "{json}");
    assert!(json.contains("\"schema\": 1"), "{json}");
    assert!(json.contains("\"id\": \"firefox\""), "{json}");
    assert!(json.contains("\"/etc/pacman.conf.pacnew\""), "{json}");
}

fn broken(now: SystemTime) -> Status {
    let mut broken = status(now);
    broken.aur_updates = Err(ProbeError::Failed(
        "paru -Qua exit status: 1: error: failed to initialize alpm".to_owned(),
    ));
    broken
}

/// `reason` is the error object's kebab-case name, so a fact that failed
/// says why in `cause`.
#[test]
fn a_failed_part_says_why_in_its_cause() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = toolbox_cli::output::with_schema(&view(&broken(now), &Ok(false), now));
    let aur = &shown["aur_updates"];

    assert_eq!(aur["state"], "failed");
    assert!(
        aur["cause"]
            .as_str()
            .is_some_and(|cause| cause.contains("alpm")),
        "{aur}"
    );
    assert!(aur.get("reason").is_none(), "{aur}");
}

#[test]
fn status_is_a_titled_section_of_facts() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = text(
        &view(&status(now), &Ok(false), now),
        Paint::new(Style::Plain),
        false,
    );

    assert!(shown.starts_with("Status\n\n"), "{shown}");
    assert!(
        shown.contains("\n  Updates         2 packages\n"),
        "{shown}"
    );
    assert!(!shown.contains("::"), "{shown}");
}

#[test]
fn facts_that_could_not_be_read_come_after_the_readable_ones() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = text(
        &view(&status(now), &Ok(false), now),
        Paint::new(Style::Plain),
        false,
    );

    let unread = shown.find("AUR updates").unwrap();
    let restart = shown.find("Restart").unwrap();
    assert!(restart < unread, "{shown}");
}

#[test]
fn each_health_finding_has_a_line_of_its_own() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = status(now);
    found.health = Ok(vec![
        HealthIssue {
            level: HealthLevel::Low,
            message: "The keyring is old.".to_owned(),
            fix_available: true,
        },
        HealthIssue {
            level: HealthLevel::Critical,
            message: "The disk is full.".to_owned(),
            fix_available: false,
        },
    ]);

    let shown = text(
        &view(&found, &Ok(false), now),
        Paint::new(Style::Plain),
        false,
    );

    assert!(shown.contains("2 findings"), "{shown}");
    assert!(shown.contains("\n  low: The keyring is old.\n"), "{shown}");
    assert!(
        shown.contains("\n  critical: The disk is full.\n"),
        "{shown}"
    );
}

#[test]
fn a_part_that_failed_says_so_in_short_and_hides_the_programs_words() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = text(
        &view(&broken(now), &Ok(false), now),
        Paint::new(Style::Plain),
        false,
    );

    assert!(
        shown.contains("error: aur updates could not be read\n"),
        "{shown}"
    );
    assert!(!shown.contains("alpm"), "{shown}");
    assert!(shown.contains("hint: add --verbose"), "{shown}");
}

#[test]
fn verbose_shows_the_programs_own_words() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = text(
        &view(&broken(now), &Ok(false), now),
        Paint::new(Style::Plain),
        true,
    );

    assert!(shown.contains("failed to initialize alpm"), "{shown}");
    assert!(!shown.contains("hint:"), "{shown}");
}

#[test]
fn several_failures_share_one_hint() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = broken(now);
    found.updates = Err(ProbeError::Failed(
        "checkupdates ==> ERROR: Cannot find the fakeroot binary".to_owned(),
    ));

    let shown = text(
        &view(&found, &Ok(false), now),
        Paint::new(Style::Plain),
        false,
    );

    assert!(
        shown.contains("error: updates could not be read\n"),
        "{shown}"
    );
    assert_eq!(shown.matches("hint:").count(), 1, "{shown}");
    assert!(!shown.contains("fakeroot"), "{shown}");
}

fn calm(now: SystemTime) -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(Vec::new()),
        merges: Ok(Vec::new()),
        last_update: Ok(Some(now - DAY * 40)),
        health: Ok(Vec::new()),
    }
}

fn finding(level: HealthLevel) -> HealthIssue {
    HealthIssue {
        level,
        message: "A finding.".to_owned(),
        fix_available: false,
    }
}

#[test]
fn the_brief_line_names_each_thing_to_do() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = view(&status(now), &Ok(true), now);

    assert_eq!(
        brief(&shown),
        "2 updates, 1 file to merge, 1 low issue, restart needed"
    );
    assert!(needs_attention(&shown));
}

#[test]
fn nothing_to_do_is_all_good() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = view(&calm(now), &Ok(false), now);

    assert_eq!(brief(&shown), "all good");
    assert!(!needs_attention(&shown));
}

#[test]
fn the_brief_line_counts_one_and_many() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = calm(now);
    found.updates = Ok(vec![update("linux")]);
    found.aur_updates = Ok(vec![update("paru"), update("yay")]);
    found.merges = Ok(vec![PathBuf::from("/a.pacnew"), PathBuf::from("/b.pacnew")]);
    found.health = Ok(vec![
        finding(HealthLevel::Low),
        finding(HealthLevel::Critical),
        finding(HealthLevel::High),
        finding(HealthLevel::Low),
    ]);

    assert_eq!(
        brief(&view(&found, &Ok(false), now)),
        "1 update, 2 AUR updates, 2 files to merge, 1 critical issue, 1 high issue, 2 low issues"
    );
}

#[test]
fn info_findings_need_no_attention() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = calm(now);
    found.health = Ok(vec![finding(HealthLevel::Info)]);
    let shown = view(&found, &Ok(false), now);

    assert_eq!(brief(&shown), "all good");
    assert!(!needs_attention(&shown));
}

#[test]
fn parts_that_could_not_be_read_are_left_out() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let mut found = calm(now);
    found.updates = Err(ProbeError::Failed("no network".to_owned()));
    found.aur_updates = Err(ProbeError::Unavailable);
    let shown = view(&found, &Err(ProbeError::Unavailable), now);

    assert_eq!(brief(&shown), "all good");
    assert!(!needs_attention(&shown));
}

#[test]
fn a_pending_restart_alone_needs_attention() {
    let now = SystemTime::UNIX_EPOCH + DAY * 100;
    let shown = view(&calm(now), &Ok(true), now);

    assert_eq!(brief(&shown), "restart needed");
    assert!(needs_attention(&shown));
}
