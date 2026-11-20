use std::collections::BTreeSet;

use toolbox_plan::{ConfigText, Fingerprint, MergeChoice, MergeCopy, Resolution};

use super::{ConfigMerges, MergeCopies};
use crate::domain::{Domain as _, Summary};
use crate::test_support::{commands, views};
use crate::{ChangeSet, Snapshot, SummaryCode, plan};

fn copy(path: &str) -> MergeCopy {
    MergeCopy::new(path).unwrap()
}

fn choice(resolution: Resolution) -> MergeChoice {
    MergeChoice {
        resolution,
        copy: Fingerprint::of(b"new"),
        current: Some(Fingerprint::of(b"old")),
    }
}

fn waiting(paths: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(MergeCopies(
        paths.iter().map(|path| copy(path)).collect::<BTreeSet<_>>(),
    ));
    snapshot
}

#[test]
fn a_copy_that_is_gone_needs_nothing() {
    let state = waiting(&["/etc/pacman.conf.pacnew"]);
    let mut changes = ChangeSet::default();

    changes.want::<ConfigMerges>(
        copy("/etc/locale.gen.pacnew"),
        choice(Resolution::KeepCurrent),
        &state,
    );

    assert!(changes.is_empty());
}

#[test]
fn an_older_save_is_never_queued_to_take_its_place() {
    let state = waiting(&["/etc/locale.gen.pacsave.1"]);
    let mut changes = ChangeSet::default();

    changes.want::<ConfigMerges>(
        copy("/etc/locale.gen.pacsave.1"),
        choice(Resolution::TakeCopy),
        &state,
    );

    assert!(changes.is_empty());
}

#[test]
fn one_snapshot_goes_before_every_merge() {
    let state = waiting(&["/etc/pacman.conf.pacnew", "/etc/locale.gen.pacsave"]);
    let mut changes = ChangeSet::default();

    changes.want::<ConfigMerges>(
        copy("/etc/pacman.conf.pacnew"),
        choice(Resolution::Merged {
            text: ConfigText::new("merged\n").unwrap(),
        }),
        &state,
    );
    changes.want::<ConfigMerges>(
        copy("/etc/locale.gen.pacsave"),
        choice(Resolution::KeepCurrent),
        &state,
    );

    let why =
        "  # only when pacman lists the file as a backup file and neither file changed meanwhile";

    assert_eq!(
        commands(&plan(&changes)),
        [
            "snapper --no-dbus -c root create --cleanup-algorithm number --description 'Garuda Toolbox: before merging configuration files', where snapper has a configuration of /  # so the merges can be rolled back with snapper".to_owned(),
            format!("rm /etc/locale.gen.pacsave{why}"),
            format!("write /etc/pacman.conf (7 bytes, merged with /etc/pacman.conf.pacnew), the old one kept as /var/lib/garuda-toolbox/backups/etc/pacman.conf, then rm /etc/pacman.conf.pacnew{why}"),
        ]
    );
}

#[test]
fn each_change_names_the_copy_and_its_file() {
    let state = waiting(&["/etc/pacman.conf.pacnew"]);
    let mut changes = ChangeSet::default();

    changes.want::<ConfigMerges>(
        copy("/etc/pacman.conf.pacnew"),
        choice(Resolution::TakeCopy),
        &state,
    );

    assert_eq!(
        views(&changes),
        [(
            "config-merges",
            "/etc/pacman.conf.pacnew".to_owned(),
            Summary::pair(
                SummaryCode::ConfigMergeTake,
                "/etc/pacman.conf.pacnew",
                "/etc/pacman.conf"
            )
        )]
    );
}

/// A `.pacnew` is pacman's copy and only kept, the saved ones are discarded.
#[test]
fn keeping_the_file_removes_a_saved_copy_as_a_discard() {
    let codes: Vec<(&str, SummaryCode)> = [
        "/etc/pacman.conf.pacnew",
        "/etc/pacman.conf.pacsave",
        "/etc/pacman.conf.pacsave.2",
        "/etc/pacman.conf.pacorig",
    ]
    .into_iter()
    .map(|path| {
        let summary = ConfigMerges::summary(&copy(path), &choice(Resolution::KeepCurrent));

        (path, summary.code)
    })
    .collect();

    assert_eq!(
        codes,
        [
            ("/etc/pacman.conf.pacnew", SummaryCode::ConfigMergeKeep),
            ("/etc/pacman.conf.pacsave", SummaryCode::ConfigMergeDiscard),
            (
                "/etc/pacman.conf.pacsave.2",
                SummaryCode::ConfigMergeDiscard
            ),
            ("/etc/pacman.conf.pacorig", SummaryCode::ConfigMergeDiscard),
        ]
    );
}
