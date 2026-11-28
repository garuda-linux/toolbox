use egui::Context;
use toolbox_catalog::LoadProblem;
use toolbox_core::features::RefusalText as _;
use toolbox_core::features::configs::Row;
use toolbox_core::features::configs::session::{
    EditedFile, Phase, Problem, ProblemCode, Session, Stage,
};

use crate::i18n::Arg;
use crate::{catalog_texts, i18n};

pub fn problem(ctx: &Context, code: ProblemCode, path: &str) -> String {
    code.text_id().map_or_else(String::new, |id| {
        i18n::text_with(ctx, id, &[("path", Arg::from(path))]).into_owned()
    })
}

/// The viewer shows a file instead of editing it, so a few problems read
/// differently there.
pub(super) fn view_problem(ctx: &Context, code: ProblemCode, path: &str) -> String {
    let id = match code {
        ProblemCode::TooLarge => "configs-view-too-large",
        ProblemCode::NotText => "configs-view-not-text",
        ProblemCode::Busy => "configs-view-busy",
        ProblemCode::Dismissed
        | ProblemCode::Denied
        | ProblemCode::NoAgent
        | ProblemCode::HelperMissing
        | ProblemCode::Broken
        | ProblemCode::Missing
        | ProblemCode::Link
        | ProblemCode::Dangling
        | ProblemCode::Outside
        | ProblemCode::NotRegular
        | ProblemCode::NoEditor
        | ProblemCode::EditorFailed
        | ProblemCode::WriteFailed
        | ProblemCode::CopyFailed
        | ProblemCode::ReadFailed
        | ProblemCode::NoHome
        | ProblemCode::Cancelled
        | ProblemCode::HelperStart
        | ProblemCode::HelperTimedOut
        | ProblemCode::HelperTooLong
        | ProblemCode::HelperPkexec
        | ProblemCode::HelperEnded
        | ProblemCode::HelperUnreadable
        | ProblemCode::HelperStopped
        | ProblemCode::Failed => return problem(ctx, code, path),
    };

    i18n::text_with(ctx, id, &[("path", Arg::from(path))]).into_owned()
}

/// The review keeps the edit after a failed write, which its text says.
pub(super) fn review_problem(ctx: &Context, code: ProblemCode, path: &str) -> String {
    match code {
        ProblemCode::WriteFailed => i18n::text_with(
            ctx,
            "configs-review-write-failed",
            &[("path", Arg::from(path))],
        )
        .into_owned(),
        ProblemCode::Dismissed
        | ProblemCode::Denied
        | ProblemCode::NoAgent
        | ProblemCode::HelperMissing
        | ProblemCode::Broken
        | ProblemCode::Missing
        | ProblemCode::Link
        | ProblemCode::Dangling
        | ProblemCode::Outside
        | ProblemCode::NotRegular
        | ProblemCode::TooLarge
        | ProblemCode::NotText
        | ProblemCode::NoEditor
        | ProblemCode::Busy
        | ProblemCode::EditorFailed
        | ProblemCode::CopyFailed
        | ProblemCode::ReadFailed
        | ProblemCode::NoHome
        | ProblemCode::Cancelled
        | ProblemCode::HelperStart
        | ProblemCode::HelperTimedOut
        | ProblemCode::HelperTooLong
        | ProblemCode::HelperPkexec
        | ProblemCode::HelperEnded
        | ProblemCode::HelperUnreadable
        | ProblemCode::HelperStopped
        | ProblemCode::Failed => problem(ctx, code, path),
    }
}

/// The system's own words below a problem, when its sentence does not
/// say it already.
pub fn detail(problem: &Problem) -> Option<&str> {
    let shown = !problem.detail.is_empty() && !restated(problem.code);
    shown.then_some(problem.detail.as_str())
}

/// The problems whose sentence already says what the system found, so its
/// own message would repeat it.
const fn restated(code: ProblemCode) -> bool {
    match code {
        ProblemCode::Missing
        | ProblemCode::Link
        | ProblemCode::Dangling
        | ProblemCode::Outside
        | ProblemCode::NotRegular
        | ProblemCode::TooLarge
        | ProblemCode::NotText => true,
        ProblemCode::Dismissed
        | ProblemCode::Denied
        | ProblemCode::NoAgent
        | ProblemCode::HelperMissing
        | ProblemCode::Broken
        | ProblemCode::NoEditor
        | ProblemCode::Busy
        | ProblemCode::EditorFailed
        | ProblemCode::WriteFailed
        | ProblemCode::CopyFailed
        | ProblemCode::ReadFailed
        | ProblemCode::NoHome
        | ProblemCode::Cancelled
        | ProblemCode::HelperStart
        | ProblemCode::HelperTimedOut
        | ProblemCode::HelperTooLong
        | ProblemCode::HelperPkexec
        | ProblemCode::HelperEnded
        | ProblemCode::HelperUnreadable
        | ProblemCode::HelperStopped
        | ProblemCode::Failed => false,
    }
}

/// What the page says while an edit runs or after it ended. `None`
/// without an edit, or while it waits in the review dialog.
pub fn status(ctx: &Context, session: &Session) -> Option<String> {
    let path = path(session.path());
    let id = match session.phase() {
        Phase::Idle => return None,
        Phase::Failed => {
            return session
                .problem()
                .map(|failed| problem(ctx, failed.code, &path));
        }
        Phase::Reading => "configs-status-reading",
        Phase::Starting(Stage::Editor) | Phase::Running(Stage::Editor) => "configs-status-editing",
        Phase::Starting(Stage::Write) | Phase::Running(Stage::Write) => "configs-status-writing",
        Phase::Comparing => "configs-status-comparing",
        Phase::Review => "configs-status-review",
        Phase::Saved => "configs-status-saved",
        Phase::Unchanged => "configs-status-unchanged",
    };

    let editor = session
        .editor()
        .map(|editor| editor.program().name())
        .unwrap_or_default();

    let backup = session.backup().unwrap_or_default();
    let words = [
        ("path", Arg::from(path.as_str())),
        ("editor", Arg::from(editor)),
        ("backup", Arg::from(backup.as_str())),
    ];

    Some(i18n::text_with(ctx, id, &words).into_owned())
}

/// The one status line of a row: why it is not edited, that it is only
/// shown, that saving needs a password, or where its link leads.
pub(super) fn row_status(ctx: &Context, row: &Row) -> String {
    let id = match row.refusal {
        "" if row.read_only => "configs-read-only",
        "" => "",
        "dangling" => "configs-not-edited-dangling",
        "not-regular" => "configs-not-edited-not-regular",
        _ => "configs-not-edited-outside",
    };

    if !id.is_empty() {
        return i18n::text(ctx, id).into_owned();
    }

    if row.link {
        return link(ctx, row);
    }

    String::new()
}

pub fn link(ctx: &Context, row: &Row) -> String {
    let target = row.target.as_ref().map_or("", |target| target.as_str());

    i18n::text_with(ctx, "configs-link", &[("target", Arg::from(target))]).into_owned()
}

pub fn unreadable(ctx: &Context, problem: &LoadProblem) -> (String, String) {
    let detail = problem
        .problems
        .iter()
        .map(|failed| catalog_texts::file(ctx, failed))
        .collect::<Vec<_>>()
        .join("\n");

    (i18n::text(ctx, "configs-unreadable").into_owned(), detail)
}

pub fn path(path: Option<&EditedFile>) -> String {
    path.map(EditedFile::to_string).unwrap_or_default()
}
