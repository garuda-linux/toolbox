use std::path::PathBuf;

use clap::Parser as _;
use toolbox_cli::Exit;
use toolbox_cli::args::{Cli, Command, Confirm, MergeResolveArgs, MergesCommand};
use toolbox_cli::output::Mark;
use toolbox_core::domains::{ConfigMerges, MergeCopies};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{ConfigFilePath, ConfigText, FetchRefusal, MergeCopy, Resolution};
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

/// The copy and its file are plain strings in JSON, `""` for no file.
#[test]
fn a_merge_row_as_json_keeps_copy_and_file_as_strings() {
    let checked = MergeRow {
        copy: Ok(MergeCopy::new("/etc/locale.gen.pacsave").unwrap()),
        file: Some(ConfigFilePath::new("/etc/locale.gen").unwrap()),
        kind: "saved".to_owned(),
        state: "1 change".to_owned(),
        unread: false,
    };

    let terminal_only = MergeRow {
        copy: Err(toolbox_text::plain("/etc/shadow.pacnew")),
        file: None,
        kind: String::new(),
        state: "merge it in a terminal".to_owned(),
        unread: false,
    };

    assert_eq!(
        serde_json::to_string(&checked).unwrap(),
        r#"{"copy":"/etc/locale.gen.pacsave","file":"/etc/locale.gen","kind":"saved","state":"1 change"}"#
    );
    assert_eq!(
        serde_json::to_string(&terminal_only).unwrap(),
        r#"{"copy":"/etc/shadow.pacnew","file":"","kind":"","state":"merge it in a terminal"}"#
    );
}

#[test]
fn the_list_says_how_each_copy_stands() {
    let paths: Vec<PathBuf> = format!("{PACDIFF_O}/etc/shadow.pacnew\n")
        .lines()
        .map(PathBuf::from)
        .collect();

    let row = |copy: &str, file: &str, kind: &str, state: &str, unread: bool| MergeRow {
        copy: MergeCopy::new(copy).map_err(|_invalid| toolbox_text::plain(copy)),
        file: (!file.is_empty()).then(|| ConfigFilePath::new(file).unwrap()),
        kind: kind.to_owned(),
        state: state.to_owned(),
        unread,
    };

    assert_eq!(
        rows(&paths, &lima_reads),
        [
            row(
                "/etc/locale.gen.pacsave",
                "/etc/locale.gen",
                "saved",
                "not read, only root may read it",
                true
            ),
            row(
                "/etc/pacman.conf.pacnew",
                "/etc/pacman.conf",
                "new",
                "1 change",
                false
            ),
            row(
                "/etc/shadow.pacnew",
                "",
                "",
                "merge it in a terminal",
                false
            ),
        ]
    );
}

/// A copy whose files could not be read is unknown, one that was read has
/// no mark of its own, whatever its state says.
#[test]
fn a_copy_not_read_is_marked_unknown_and_a_read_one_is_blank() {
    let paths: Vec<PathBuf> = PACDIFF_O.lines().map(PathBuf::from).collect();
    let listed_rows = rows(&paths, &lima_reads);
    let listed = super::listing(&listed_rows);
    let mark_of = |copy: &str| {
        listed
            .rows
            .iter()
            .find(|row| row.cells[0] == copy)
            .map(|row| row.mark)
    };

    assert_eq!(mark_of("/etc/locale.gen.pacsave"), Some(Mark::Unknown));
    assert_eq!(mark_of("/etc/pacman.conf.pacnew"), Some(Mark::Blank));
}

/// The Lima VM's copies: `pacman.conf` reads, the others need root.
fn lima_reads(found: &MergeCopy) -> MergePair {
    match found.as_str() {
        "/etc/pacman.conf.pacnew" => pair(found.as_str(), &newer(), Some(PACMAN_CONF)),
        _ => MergePair {
            copy: found.clone(),
            copied: Err(ReadProblem::NotRun(pkexec::Refusal::Dismissed)),
            current: Err(ReadProblem::Refused(FetchRefusal::Missing)),
        },
    }
}

/// A merge the page refuses under `--json` is one JSON Lines error on
/// standard output, with the page rule's exit code.
#[test]
fn a_refused_merge_under_json_is_one_error_line() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let same = pair("/etc/pacman.conf.pacnew", PACMAN_CONF, Some(PACMAN_CONF));
    let refused = queued(&["/etc/pacman.conf.pacnew", "--take-new"], &same).unwrap_err();
    let confirm = Confirm {
        yes: true,
        json: true,
        ..Confirm::default()
    };

    let mut out = Vec::new();
    let exit = crate::act::run(
        Err(refused),
        confirm,
        None,
        &runtime,
        &mut crate::out::Out::new(&mut out, crate::out::Settings::default()),
    );

    let text = String::from_utf8(out).unwrap();
    assert_eq!(exit, Exit::PageRule);
    assert!(
        text.starts_with(r#"{"schema":1,"event":"error","message":"#),
        "{text}"
    );
    assert_eq!(text.lines().count(), 1, "{text}");
}
