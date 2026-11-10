use toolbox_cli::Exit;
use toolbox_core::domains::{NewGroups, Packages};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Queued, Snapshot, Summary, Toggle};
use toolbox_services::queue::edit::Removal;

use super::{ChangeRow, listing, pick, rows};

fn queue() -> ChangeSet {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<Packages>("firefox".parse().unwrap(), Toggle::On, &snapshot);
    changes.want::<Packages>("cups".parse().unwrap(), Toggle::On, &snapshot);
    changes.want::<NewGroups>("cups".parse().unwrap(), Queued, &snapshot);
    changes
}

fn worded(summary: &Summary) -> String {
    format!("{} {}", summary.code.as_str(), summary.values.join(" "))
}

fn shown() -> Vec<ChangeRow> {
    rows(&queue().views(), worded)
}

#[test]
fn every_change_has_its_number_in_plan_order() {
    let rows = shown();

    let numbers: Vec<usize> = rows.iter().map(|row| row.number).collect();
    assert_eq!(numbers, [1, 2, 3]);
    assert!(rows.iter().all(|row| !row.change.is_empty()));
}

#[test]
fn a_number_picks_its_change() {
    let picked = pick(&shown(), "3").unwrap();

    assert_eq!(picked, [Removal::from(&shown()[2])]);
}

#[test]
fn a_name_picks_the_one_change_with_it() {
    let picked = pick(&shown(), "firefox").unwrap();

    assert_eq!(picked.len(), 1);
    assert_eq!(picked[0].key, "firefox");
}

#[test]
fn a_name_of_two_changes_wants_a_number() {
    let refused = pick(&shown(), "cups").unwrap_err();

    assert_eq!(refused.exit(), Exit::Usage);
    assert!(refused.message().contains("number"));
}

#[test]
fn an_unknown_number_or_name_is_refused_with_a_hint() {
    assert_eq!(pick(&shown(), "9").unwrap_err().exit(), Exit::Unavailable);

    let typo = pick(&shown(), "firefx").unwrap_err();
    assert_eq!(typo.exit(), Exit::Unavailable);
    assert!(typo.message().contains("firefox"));
}

#[test]
fn the_list_counts_the_changes_and_points_at_apply() {
    let rows = shown();
    let listed = listing(&rows, false);

    assert_eq!(listed.title, "Queued");
    assert_eq!(listed.summary.as_deref(), Some("3 changes"));
    assert!(listed.hint.is_some_and(|hint| hint.contains("queue apply")));

    let in_window = listing(&rows, true);
    assert!(in_window.hint.is_some_and(|hint| hint.contains("window")));
}
