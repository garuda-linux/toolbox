use std::path::PathBuf;

use clap::Parser as _;
use toolbox_cli::args::{Cli, Command, MergeResolveArgs, MergesCommand};
use toolbox_core::domains::{ConfigMerges, MergeCopies};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{ConfigText, FetchRefusal, MergeCopy, Resolution};
use toolbox_system::{MergePair, ReadProblem, pkexec};

use super::{MergeRow, onto, rows};
use crate::change::Refused;

/// `/etc/pacman.conf` of the Lima VM.
const PACMAN_CONF: &str = toolbox_system::testing::packages::PACMAN_CONF;
/// `pacdiff -o` as root in the Lima VM.
const PACDIFF_O: &str = toolbox_system::testing::status::PACDIFF_O;

fn copy(path: &str) -> MergeCopy {
    MergeCopy::new(path).unwrap()
}

fn resolve(words: &[&str]) -> MergeResolveArgs {
    let typed = ["garuda-toolbox", "merges", "resolve"]
        .into_iter()
        .chain(words.iter().copied());

    match Cli::try_parse_from(typed).unwrap().command {
        Some(Command::Merges(MergesCommand::Resolve(args))) => args,
        other => panic!("not a resolve: {other:?}"),
    }
}

/// The copies of the Lima VM waiting.
fn waiting() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(MergeCopies(PACDIFF_O.lines().map(copy).collect()));
    snapshot
}

/// The Lima VM's `pacman.conf` and a copy beside it with `new` as its text.
fn pair(path: &str, new: &str, current: Option<&str>) -> MergePair {
    MergePair {
        copy: copy(path),
        copied: Ok(ConfigText::new(new).unwrap()),
        current: Ok(current.map(|text| ConfigText::new(text).unwrap())),
    }
}

/// The capture with the number of parallel downloads changed.
fn newer() -> String {
    PACMAN_CONF.replace("ParallelDownloads = 5", "ParallelDownloads = 10")
}

fn queued(words: &[&str], pair: &MergePair) -> Result<ChangeSet, Refused> {
    let mut changes = ChangeSet::default();

    onto(&resolve(words), pair, &waiting(), &mut changes)?;
    Ok(changes)
}

#[test]
fn keep_and_take_new_queue_the_whole_file_with_both_marks() {
    let pacnew = pair("/etc/pacman.conf.pacnew", &newer(), Some(PACMAN_CONF));
    let take = queued(&["/etc/pacman.conf.pacnew", "--take-new"], &pacnew).unwrap();
    let keep = queued(&["/etc/pacman.conf.pacnew", "--keep"], &pacnew).unwrap();
    let wish = |changes: &ChangeSet| {
        changes
            .wish::<ConfigMerges>(&copy("/etc/pacman.conf.pacnew"))
            .cloned()
            .unwrap()
    };

    assert_eq!(wish(&take).resolution, Resolution::TakeCopy);
    assert_eq!(
        wish(&take).current,
        Some(toolbox_plan::Fingerprint::of(PACMAN_CONF.as_bytes()))
    );
    assert_eq!(wish(&keep).resolution, Resolution::KeepCurrent);
}

#[test]
fn keep_and_take_new_exclude_each_other_and_one_is_needed() {
    let argv = |words: &[&'static str]| {
        [
            "garuda-toolbox",
            "merges",
            "resolve",
            "/etc/pacman.conf.pacnew",
        ]
        .into_iter()
        .chain(words.iter().copied())
        .collect::<Vec<_>>()
    };

    Cli::try_parse_from(argv(&[])).unwrap_err();
    Cli::try_parse_from(argv(&["--keep", "--take-new"])).unwrap_err();
    Cli::try_parse_from(["garuda-toolbox", "merges", "show", "/etc/shadow.pacnew"]).unwrap_err();
}

#[test]
fn an_older_save_or_an_equal_copy_only_goes() {
    let older = pair("/etc/locale.gen.pacsave.1", "x\n", Some("y\n"));
    let same = pair("/etc/pacman.conf.pacnew", PACMAN_CONF, Some(PACMAN_CONF));
    let mut snapshot = waiting();

    snapshot.insert(MergeCopies(
        [
            copy("/etc/locale.gen.pacsave.1"),
            copy("/etc/pacman.conf.pacnew"),
        ]
        .into(),
    ));

    for (path, pair) in [
        ("/etc/locale.gen.pacsave.1", &older),
        ("/etc/pacman.conf.pacnew", &same),
    ] {
        let refused = onto(
            &resolve(&[path, "--take-new"]),
            pair,
            &snapshot,
            &mut ChangeSet::default(),
        );

        assert!(
            matches!(&refused, Err(Refused::Rule(why)) if why.ends_with("use --keep")),
            "{path}: {refused:?}"
        );
    }
}

#[test]
fn a_copy_that_does_not_wait_is_refused() {
    let gone = pair("/etc/mkinitcpio.conf.pacnew", "x\n", Some("y\n"));
    let refused = queued(&["/etc/mkinitcpio.conf.pacnew", "--keep"], &gone);

    assert!(matches!(refused, Err(Refused::Unknown(_))), "{refused:?}");
}

#[test]
fn a_copy_without_its_file_takes_its_place() {
    let pacnew = pair("/etc/pacman.conf.pacnew", &newer(), None);
    let changes = queued(&["/etc/pacman.conf.pacnew", "--take-new"], &pacnew).unwrap();
    let wish = changes
        .wish::<ConfigMerges>(&copy("/etc/pacman.conf.pacnew"))
        .unwrap();

    assert_eq!(wish.current, None);
}

#[test]
fn the_list_says_how_each_copy_stands() {
    let paths: Vec<PathBuf> = format!("{PACDIFF_O}/etc/shadow.pacnew\n")
        .lines()
        .map(PathBuf::from)
        .collect();

    let read = |found: &MergeCopy| match found.as_str() {
        "/etc/pacman.conf.pacnew" => pair(found.as_str(), &newer(), Some(PACMAN_CONF)),
        _ => MergePair {
            copy: found.clone(),
            copied: Err(ReadProblem::NotRun(pkexec::Refusal::Dismissed)),
            current: Err(ReadProblem::Refused(FetchRefusal::Missing)),
        },
    };

    let row = |copy: &str, file: &str, kind: &str, state: &str| MergeRow {
        copy: copy.to_owned(),
        file: file.to_owned(),
        kind: kind.to_owned(),
        state: state.to_owned(),
    };

    assert_eq!(
        rows(&paths, &read),
        [
            row(
                "/etc/locale.gen.pacsave",
                "/etc/locale.gen",
                "saved",
                "not read, only root may read it"
            ),
            row(
                "/etc/pacman.conf.pacnew",
                "/etc/pacman.conf",
                "new",
                "1 change"
            ),
            row("/etc/shadow.pacnew", "", "", "merge it in a terminal"),
        ]
    );
}
