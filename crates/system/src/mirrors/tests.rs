use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::{
    MirrorRanking as _, RANKING_DEADLINE, RankProblem, RateMirrors, StartProblem, rank_within,
    stderr_tail,
};
use crate::test_support::runnable_script;

/// The file `rate-mirrors --save <file> chaotic-aur` 0.33.0 saved in the
/// Lima VM, 28 servers below its log.
const RATED: &str = toolbox_plan::testing::mirrorlist::RATE_MIRRORS_CHAOTIC;
/// `rate-mirrors --save <file> chaotic-aur` 0.33.0 in the Lima VM without a
/// network.
const OFFLINE_STDOUT: &str = include_str!("fixtures/image/rate-mirrors-offline.stdout");
const OFFLINE_STDERR: &str = include_str!("fixtures/image/rate-mirrors-offline.stderr");
const OFFLINE_CODE: &str = include_str!("fixtures/image/rate-mirrors-offline.code");

/// A stand-in for `rate-mirrors`: `--save <file> chaotic-aur`, so the
/// file is the second argument.
fn stand_in(folder: &Path, body: &str) -> PathBuf {
    let script = folder.join("rate-mirrors");

    runnable_script(&script, body);
    script
}

/// A stand-in that saves `ranked` as its result.
fn saving(folder: &Path, ranked: &str) -> PathBuf {
    let result = folder.join("ranked");

    fs::write(&result, ranked).unwrap();
    stand_in(folder, &format!("cp '{}' \"$2\"", result.display()))
}

#[test]
fn a_ranked_list_becomes_a_mirrorlist() {
    let folder = tempfile::tempdir().unwrap();
    let program = saving(folder.path(), RATED);

    let mirrorlist = rank(&program).unwrap();

    assert_eq!(mirrorlist.as_str(), RATED);
    assert_eq!(mirrorlist.servers(), 28);
}

/// rate-mirrors of the Lima image without a network, replayed.
#[test]
fn a_failing_ranking_names_its_status_and_the_end_of_its_complaint() {
    let folder = tempfile::tempdir().unwrap();
    let said = folder.path().join("said");
    let complained = folder.path().join("complained");

    fs::write(&said, OFFLINE_STDOUT).unwrap();
    fs::write(&complained, OFFLINE_STDERR).unwrap();

    let program = stand_in(
        folder.path(),
        &format!(
            "cat '{}'; cat '{}' >&2; exit {}",
            said.display(),
            complained.display(),
            OFFLINE_CODE.trim()
        ),
    );

    let problem = rank(&program).unwrap_err();

    assert!(
        matches!(&problem, RankProblem::Failed(reason) if reason.contains("exit status: 1") && reason.contains("failed to connect to https://gitlab.com/")),
        "{problem:?}"
    );
}

#[test]
fn a_missing_program_did_not_start() {
    let problem = rank(Path::new("/nonexistent/rate-mirrors")).unwrap_err();

    assert_eq!(
        problem,
        RankProblem::NotStarted(StartProblem::Missing),
        "{problem:?}"
    );
}

/// The Lima result with `Include = /etc/passwd` appended.
#[test]
fn a_result_that_is_no_mirrorlist_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let program = saving(folder.path(), &format!("{RATED}Include = /etc/passwd\n"));

    assert!(matches!(
        rank(&program).unwrap_err(),
        RankProblem::Invalid(_)
    ));
}

#[test]
fn no_saved_file_is_unreadable() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(folder.path(), "exit 0");

    assert!(matches!(
        rank(&program).unwrap_err(),
        RankProblem::Unreadable(_)
    ));
}

#[test]
fn a_ranking_past_its_deadline_is_stopped() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(folder.path(), "sleep 60");
    let started = Instant::now();

    let problem = rank_within(&program, Duration::from_millis(200), &Arc::default()).unwrap_err();

    assert_eq!(problem, RankProblem::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn a_cancel_stops_the_ranking_at_once() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(folder.path(), "sleep 60");
    let cancel = Arc::new(AtomicBool::new(false));
    let setter = Arc::clone(&cancel);
    let started = Instant::now();
    std::thread::Builder::new()
        .name("cancel-timer".to_owned())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            setter.store(true, Ordering::Relaxed);
        })
        .unwrap();

    let problem = rank_within(&program, RANKING_DEADLINE, &cancel).unwrap_err();

    assert_eq!(problem, RankProblem::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(10));
}

fn rank(program: &Path) -> Result<toolbox_plan::Mirrorlist, RankProblem> {
    rank_within(program, RANKING_DEADLINE, &Arc::default())
}

/// The Lima result with its log stands in for a long complaint.
#[test]
fn only_the_end_of_a_long_complaint_is_kept() {
    let tail = stderr_tail(RATED.as_bytes());

    assert_eq!(tail.len(), 2048);
    assert!(RATED.ends_with(&tail));
    assert_eq!(stderr_tail(b"short"), "short");
}

#[test]
fn the_system_ranking_runs_its_program() {
    let folder = tempfile::tempdir().unwrap();
    let program = saving(folder.path(), RATED);

    let mirrorlist = RateMirrors::new(program).rank(&Arc::default()).unwrap();

    assert_eq!(mirrorlist.servers(), 28);
}
