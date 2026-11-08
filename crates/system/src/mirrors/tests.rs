use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{
    MirrorRanking as _, RANKING_DEADLINE, RankProblem, RateMirrors, rank_within, stderr_tail,
};
use crate::testing::runnable_script;

/// A stand-in for `rate-mirrors`: `--save <file> chaotic-aur`, so the
/// file is the second argument.
fn stand_in(folder: &Path, body: &str) -> PathBuf {
    let script = folder.join("rate-mirrors");

    runnable_script(&script, body);
    script
}

#[test]
fn a_ranked_list_becomes_a_mirrorlist() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(
        folder.path(),
        "printf 'Server = https://cdn-mirror.chaotic.cx/$repo/$arch\\n' > \"$2\"",
    );

    let mirrorlist = rank(&program).unwrap();

    assert!(mirrorlist.as_str().contains("cdn-mirror.chaotic.cx"));
}

#[test]
fn a_failing_ranking_names_its_status_and_the_end_of_its_complaint() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(folder.path(), "echo 'no mirror answered' >&2; exit 3");

    let problem = rank(&program).unwrap_err();

    assert!(
        matches!(&problem, RankProblem::Failed(reason) if reason.contains("exit status: 3") && reason.contains("no mirror answered")),
        "{problem:?}"
    );
}

#[test]
fn a_missing_program_did_not_start() {
    let problem = rank(Path::new("/nonexistent/rate-mirrors")).unwrap_err();

    assert!(matches!(problem, RankProblem::NotStarted(_)), "{problem:?}");
}

#[test]
fn a_result_that_is_no_mirrorlist_is_refused() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(folder.path(), "echo 'Include = /etc/passwd' > \"$2\"");

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

    let problem = rank_within(&program, Duration::from_millis(200)).unwrap_err();

    assert_eq!(problem, RankProblem::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(10));
}

fn rank(program: &Path) -> Result<toolbox_plan::Mirrorlist, RankProblem> {
    rank_within(program, RANKING_DEADLINE)
}

#[test]
fn only_the_end_of_a_long_complaint_is_kept() {
    let long = format!("{}the reason", "x".repeat(10_000));
    let tail = stderr_tail(long.as_bytes());

    assert_eq!(tail.len(), 2048);
    assert!(tail.ends_with("the reason"));
    assert_eq!(stderr_tail(b"short"), "short");
}

#[test]
fn the_system_ranking_runs_its_program() {
    let folder = tempfile::tempdir().unwrap();
    let program = stand_in(
        folder.path(),
        "printf 'Server = https://cdn-mirror.chaotic.cx/$repo/$arch\\n' > \"$2\"",
    );

    let mirrorlist = RateMirrors::new(program).rank().unwrap();

    assert_eq!(mirrorlist.servers(), 1);
}
