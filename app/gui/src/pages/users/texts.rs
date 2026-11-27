use egui::Context;
use toolbox_core::features::RefusalText as _;
use toolbox_core::features::users::input::InputProblem;
use toolbox_core::features::users::run::{PageRun, RunKind};
use toolbox_plan::{InvalidFullName, InvalidPassword};
use toolbox_services::locks::Locks;
use toolbox_system::pkexec::Refusal;

use crate::i18n;
use crate::i18n::Arg;

/// Why a change or a typed value was refused. `name` names the account
/// or group a refusal is about.
pub fn problem(ctx: &Context, problem: InputProblem, name: &str) -> String {
    problem.text_id().map_or_else(String::new, |id| {
        i18n::text_with(ctx, id, &[("name", Arg::from(name))]).into_owned()
    })
}

pub(super) fn full_name_problem(ctx: &Context, rule: InvalidFullName) -> String {
    InputProblem::FullName(rule)
        .text_id()
        .map_or_else(String::new, |id| i18n::text(ctx, id).into_owned())
}

/// What is wrong with a typed password.
pub(super) fn password_problem(ctx: &Context, problem: InvalidPassword) -> String {
    i18n::text(ctx, password_id(problem)).into_owned()
}

pub(super) fn mismatch(ctx: &Context) -> String {
    i18n::text(ctx, "users-password-mismatch").into_owned()
}

/// The ids are literals, so grep and the catalog checks see each one.
const fn password_id(problem: InvalidPassword) -> &'static str {
    match problem {
        InvalidPassword::Empty => "users-password-empty",
        InvalidPassword::TooLong { max: _ } => "users-password-too-long",
        InvalidPassword::ControlCharacter => "users-password-control-character",
    }
}

/// The note on why the lock flags are missing, `None` once they are known
/// or before the page asked.
pub fn locks(ctx: &Context, locks: &Locks) -> Option<String> {
    match locks {
        Locks::NotAsked | Locks::Known(_) => None,
        Locks::Loading => Some(i18n::text(ctx, "users-locks-loading").into_owned()),
        Locks::Refused(refusal) => Some(i18n::text(ctx, refused_lock_id(*refusal)).into_owned()),
        Locks::Unknown => Some(i18n::text(ctx, "users-locks-unknown").into_owned()),
    }
}

/// The ids are literals, so grep and the catalog checks see each one.
pub const fn refused_lock_id(refusal: Refusal) -> &'static str {
    match refusal {
        Refusal::Dismissed => "users-locks-dismissed",
        Refusal::Denied => "users-locks-denied",
        Refusal::NoAgent => "users-locks-no-agent",
        Refusal::HelperMissing => "users-locks-helper-missing",
        Refusal::Broken => "users-locks-broken",
    }
}

pub fn run(ctx: &Context, run: &PageRun) -> Option<String> {
    let id = match run {
        PageRun::None => return None,
        PageRun::Sent(kind, _) | PageRun::Running(kind, _) => match kind {
            RunKind::Password => "users-run-password-running",
            RunKind::NewAccount => "users-run-new-account-running",
        },
        PageRun::Done(kind, _) => match kind {
            RunKind::Password => "users-run-password-done",
            RunKind::NewAccount => "users-run-new-account-done",
        },
        PageRun::Failed(kind, _) => match kind {
            RunKind::Password => "users-run-password-failed",
            RunKind::NewAccount => "users-run-new-account-failed",
        },
    };

    Some(i18n::text_with(ctx, id, &[("user", Arg::from(run.user()))]).into_owned())
}

pub fn list(ctx: &Context, names: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let separator = i18n::text(ctx, "users-list-separator");
    names
        .into_iter()
        .map(|name| name.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(&separator)
}
