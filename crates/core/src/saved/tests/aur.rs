//! The AUR wish as it is saved. An order is read by the app, never written
//! to a file or taken from one.

use serde_json::{Value, json};
use toolbox_plan::BuildRun;
use toolbox_state::aur::order::parse_order;
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;

use super::{RestoreProblem, SavedChanges};
use crate::domains::{AurPackages, AurWish, BuildOrder};
use crate::test_support::package;
use crate::{ChangeSet, Snapshot, planned};

fn octopi_built() -> AurWish {
    AurWish::Built(BuildOrder {
        order: parse_order(OCTOPI_GIT_STDOUT).unwrap(),
        run: BuildRun::new("1a2b").unwrap(),
        installed: std::iter::once(package("alpm_octopi_utils-git")).collect(),
    })
}

/// A saved file with `built` for octopi-git, whose order names one more
/// repository package than paru printed.
fn saved_file_with_an_extra_repository_package() -> String {
    let mut order = serde_json::to_value(parse_order(OCTOPI_GIT_STDOUT).unwrap()).unwrap();
    if let Some(repo) = order.get_mut("repo").and_then(Value::as_array_mut) {
        repo.push(json!({"role": "Target", "name": "evil-package"}));
    }

    json!({
        "version": 6,
        "changes": [{
            "domain": "aur-packages",
            "item": "octopi-git",
            "wish": {"built": {"order": order, "run": "1a2b", "installed": []}},
        }],
    })
    .to_string()
}

#[test]
fn paru_and_waiting_wishes_are_saved_and_read_back_as_they_were() {
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package("yay-bin"), AurWish::Paru, &Snapshot::default());
    changes.want::<AurPackages>(package("paru-bin"), AurWish::Waiting, &Snapshot::default());
    let saved = changes.to_saved();
    let text = serde_json::to_string(&saved).unwrap();

    let read = SavedChanges::parse(&text).unwrap();
    let mut restored = ChangeSet::default();
    let domains = crate::domains::all().unwrap();
    let problems = domains.restore(&mut restored, &read, &Snapshot::default());

    assert_eq!(problems, []);
    assert_eq!(restored.to_saved(), saved);
}

#[test]
fn a_built_wish_is_saved_as_waiting_and_its_order_is_not_written() {
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package("octopi-git"), octopi_built(), &Snapshot::default());

    let text = serde_json::to_string_pretty(&changes.to_saved()).unwrap();

    assert!(text.contains(r#""wish": "waiting""#), "{text}");
    assert!(!text.contains("alpm_octopi_utils-git"), "{text}");
    assert!(!text.contains("order"), "{text}");
    assert!(!text.contains("built"), "{text}");
}

#[test]
fn a_built_wish_in_a_saved_file_comes_back_waiting_and_plans_no_root_step() {
    let read = SavedChanges::parse(&saved_file_with_an_extra_repository_package()).unwrap();
    let mut restored = ChangeSet::default();
    let domains = crate::domains::all().unwrap();

    let problems = domains.restore(&mut restored, &read, &Snapshot::default());

    assert_eq!(problems, []);
    assert_eq!(
        restored.wish::<AurPackages>(&package("octopi-git")),
        Some(&AurWish::Waiting)
    );
    assert_eq!(planned(&restored).0.steps(), []);
}

#[test]
fn a_saved_queue_of_version_five_restores_nothing() {
    let text =
        r#"{"version":5,"changes":[{"domain":"aur-packages","item":"yay-bin","wish":null}]}"#;

    let saved: SavedChanges = serde_json::from_str(text).unwrap();
    let mut changes = ChangeSet::default();

    let domains = crate::domains::all().unwrap();
    let problems = domains.restore(&mut changes, &saved, &Snapshot::default());

    assert_eq!(problems, [RestoreProblem::Version(5)]);
    assert!(!changes.holds("aur-packages"));
    assert_eq!(SavedChanges::parse(text), Err(RestoreProblem::Version(5)));
}
