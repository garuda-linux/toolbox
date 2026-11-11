use serde_json::json;
use toolbox_core::domains::{InstalledPackages, Packages, all};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, SAVED_VERSION, Snapshot, Toggle};
use toolbox_plan::PackageName;

use super::{Invalid, MAX_EDITS, QUEUE_EDIT_VERSION, QueueEdit, Removal};

fn package(name: &str) -> PackageName {
    name.parse().unwrap()
}

fn queued(names: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for name in names {
        changes.want::<Packages>(package(name), Toggle::On, &Snapshot::default());
    }

    changes
}

fn keys(changes: &ChangeSet) -> Vec<String> {
    changes.views().into_iter().map(|view| view.key).collect()
}

fn parse(text: &str) -> Result<QueueEdit, Invalid> {
    QueueEdit::parse(text, &all().unwrap())
}

#[test]
fn the_edit_between_two_queues_adds_and_removes_what_differs() {
    let before = queued(&["firefox", "vlc"]);
    let after = queued(&["firefox", "steam"]);

    let edit = QueueEdit::between(&before, &after);

    assert_eq!(
        edit.remove,
        [Removal {
            domain: "packages".to_owned(),
            key: "vlc".to_owned()
        }]
    );

    assert_eq!(edit.add.len(), 1);
    assert_eq!(edit.add[0].item, json!("steam"));
}

#[test]
fn an_edit_carried_out_on_another_queue_keeps_its_own_changes() {
    let edit = QueueEdit::between(&queued(&["firefox", "vlc"]), &queued(&["firefox", "steam"]));
    let mut window = queued(&["vlc", "gimp"]);

    let problems = edit.apply(&mut window, &all().unwrap(), &Snapshot::default());

    assert_eq!(problems, []);
    assert_eq!(keys(&window), ["gimp", "steam"]);
}

#[test]
fn an_added_wish_the_system_fulfils_is_not_queued() {
    let edit = QueueEdit::between(&ChangeSet::default(), &queued(&["firefox"]));
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages([package("firefox")].into()));
    let mut changes = ChangeSet::default();

    edit.apply(&mut changes, &all().unwrap(), &snapshot);

    assert!(changes.is_empty());
}

#[test]
fn an_edit_survives_its_text() {
    let edit = QueueEdit::between(&queued(&["vlc"]), &queued(&["steam"]));

    assert_eq!(parse(&edit.to_text().unwrap()), Ok(edit));
}

#[test]
fn text_that_is_no_edit_is_refused() {
    assert_eq!(parse("rm -rf /"), Err(Invalid::Shape));
    assert_eq!(
        parse(&edit_text(&json!({"remove": [], "add": [], "run": "now"}))),
        Err(Invalid::Shape)
    );
}

/// The edit's own shape and the shape of the changes it carries change
/// for different reasons, so each has its version and either refuses.
#[test]
fn an_edit_of_another_version_or_with_changes_of_another_is_refused() {
    let edit = |version: u32, saved: u32| {
        json!({"version": version, "saved_version": saved, "remove": [], "add": []}).to_string()
    };

    parse(&edit(QUEUE_EDIT_VERSION, SAVED_VERSION)).unwrap();
    assert_eq!(
        parse(&edit(QUEUE_EDIT_VERSION + 1, SAVED_VERSION)),
        Err(Invalid::Version)
    );
    assert_eq!(
        parse(&edit(QUEUE_EDIT_VERSION, SAVED_VERSION + 1)),
        Err(Invalid::Version)
    );
}

/// An edit of this build's versions with `fields` beside them.
fn edit_text(fields: &serde_json::Value) -> String {
    let mut text = json!({"version": QUEUE_EDIT_VERSION, "saved_version": SAVED_VERSION});
    if let (Some(text), Some(fields)) = (text.as_object_mut(), fields.as_object()) {
        text.extend(fields.clone());
    }

    text.to_string()
}

#[test]
fn a_change_of_an_unknown_domain_or_with_a_bad_value_is_refused() {
    let unknown = edit_text(&json!({"remove": [{"domain": "shell", "key": "x"}], "add": []}));
    let bad_package = edit_text(&json!({
        "remove": [],
        "add": [{"domain": "packages", "item": "-rf", "wish": "on"}],
    }));

    let control = edit_text(&json!({"remove": [{"domain": "packages", "key": "a\nb"}], "add": []}));

    assert_eq!(parse(&unknown), Err(Invalid::Domain));
    assert_eq!(parse(&bad_package), Err(Invalid::Change));
    assert_eq!(parse(&control), Err(Invalid::Key));
}

#[test]
fn an_edit_with_too_many_changes_is_refused() {
    let removals: Vec<_> = (0..=MAX_EDITS)
        .map(|n| json!({"domain": "packages", "key": format!("p{n}")}))
        .collect();

    let text = edit_text(&json!({"remove": removals, "add": []}));

    assert_eq!(parse(&text), Err(Invalid::TooMany));
}
