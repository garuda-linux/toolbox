use egui::Context;
use toolbox_catalog::LoadProblem;
use toolbox_core::features::configs::Row;
use toolbox_core::features::configs::session::{EditedFile, Phase, Problem, Session, Stage};

use crate::i18n;

/// The codes with words of their own. Any other is a failed read.
const PROBLEMS: [&str; 19] = [
    "dismissed",
    "denied",
    "no-agent",
    "helper-missing",
    "missing",
    "link",
    "dangling",
    "outside",
    "not-regular",
    "too-large",
    "not-text",
    "no-editor",
    "busy",
    "editor-failed",
    "write-failed",
    "copy-failed",
    "unknown",
    "read-only",
    "other",
];

/// The problems whose sentence already says what the system found, so its
/// own message would repeat it.
const RESTATED: [&str; 8] = [
    "missing",
    "link",
    "dangling",
    "outside",
    "not-regular",
    "too-large",
    "not-text",
    "read-only",
];

pub fn problem(ctx: &Context, code: &str, path: &str) -> String {
    let code = if PROBLEMS.contains(&code) {
        code
    } else {
        "other"
    };

    i18n::text_with(ctx, &format!("configs-problem-{code}"), &[("path", path)]).into_owned()
}

pub fn view_problem(ctx: &Context, code: &str, path: &str) -> String {
    match code {
        "too-large" | "not-text" | "busy" => {
            i18n::text_with(ctx, &format!("configs-view-{code}"), &[("path", path)]).into_owned()
        }
        _ => problem(ctx, code, path),
    }
}

/// The system's own words below a problem, when its sentence does not
/// say it already.
pub fn detail(problem: &Problem) -> Option<&str> {
    let shown = !problem.detail.is_empty() && !RESTATED.contains(&problem.code);
    shown.then_some(problem.detail.as_str())
}

/// What the page says while an edit runs or after it ended. `None`
/// without an edit, or while it waits in the review dialog.
pub fn status(ctx: &Context, session: &Session) -> Option<String> {
    let path = path(session.path());
    let id = match session.phase() {
        Phase::Idle => return None,
        Phase::Failed => return Some(problem(ctx, session.problem().code, &path)),
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
        ("path", path.as_str()),
        ("editor", editor),
        ("backup", backup.as_str()),
    ];

    Some(i18n::text_with(ctx, id, &words).into_owned())
}

/// The one status line of a row: why it is not edited, that it is only
/// shown, that saving needs a password, or where its link leads.
pub fn row_status(ctx: &Context, row: &Row) -> String {
    let id = match row.refusal {
        "" if row.read_only => "configs-read-only",
        "" if row.admin => "configs-admin",
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
    i18n::text_with(ctx, "configs-link", &[("target", &row.target)]).into_owned()
}

pub fn unreadable(ctx: &Context, problem: &LoadProblem) -> (String, String) {
    (
        i18n::text(ctx, "configs-unreadable").into_owned(),
        problem.detail.clone(),
    )
}

pub fn path(path: Option<&EditedFile>) -> String {
    path.map(EditedFile::to_string).unwrap_or_default()
}
