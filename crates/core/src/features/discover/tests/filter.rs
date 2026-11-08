use super::{apps, ids, offering};
use crate::features::discover::Apps;
use crate::{ChangeSet, Snapshot};

fn counts(apps: &Apps, search: &str, snapshot: &Snapshot) -> Vec<(String, usize)> {
    apps.categories_matching(search, snapshot, &ChangeSet::default())
        .iter()
        .map(|category| (category.section.id().to_owned(), category.count))
        .collect()
}

#[test]
fn a_search_counts_the_matches_of_each_category() {
    let apps = apps();
    let snapshot = Snapshot::default();

    assert_eq!(
        counts(&apps, "browser", &snapshot),
        [("internet".to_owned(), 1), ("utilities".to_owned(), 2)]
    );
    assert_eq!(
        counts(&apps, "", &snapshot),
        [("internet".to_owned(), 2), ("utilities".to_owned(), 2)],
        "without a search every offered app counts"
    );
}

#[test]
fn a_category_without_a_match_stays_with_none() {
    let apps = apps();

    assert_eq!(
        counts(&apps, "w3m", &Snapshot::default()),
        [("internet".to_owned(), 1), ("utilities".to_owned(), 0)]
    );
}

#[test]
fn a_category_the_system_offers_nothing_of_stays_hidden_in_a_search() {
    let apps = apps();

    assert_eq!(
        counts(&apps, "browser", &offering(&["firefox"])),
        [("internet".to_owned(), 1)]
    );
}

#[test]
fn all_lists_every_category_and_a_search_narrows_it() {
    let apps = apps();
    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();

    assert_eq!(
        ids(&apps.filtered(None, "", &snapshot, &changes)),
        ["firefox", "aerc", "kate", "brave-bin"]
    );
    assert_eq!(
        ids(&apps.filtered(None, "browser", &snapshot, &changes)),
        ["firefox", "kate", "brave-bin"]
    );
}

#[test]
fn a_picked_category_stays_picked_during_a_search() {
    let apps = apps();
    let snapshot = Snapshot::default();
    let changes = ChangeSet::default();

    assert_eq!(
        ids(&apps.filtered(Some("utilities"), "browser", &snapshot, &changes)),
        ["kate", "brave-bin"]
    );
    assert_eq!(
        ids(&apps.filtered(Some("utilities"), "", &snapshot, &changes)),
        ["kate", "brave-bin"]
    );
}

#[test]
fn filtered_leaves_out_what_the_system_does_not_offer() {
    let apps = apps();

    assert_eq!(
        ids(&apps.filtered(None, "", &offering(&["kate"]), &ChangeSet::default())),
        ["kate"]
    );
}
