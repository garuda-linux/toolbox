use egui::Context;
use toolbox_core::features::users::run::PageRun;
use toolbox_system::Locks;

use crate::i18n;

/// Why a change or a typed value was refused. `name` names the account
/// or group a refusal is about.
pub fn problem(ctx: &Context, code: &str, name: &str) -> String {
    let id = if code.starts_with("home-") {
        format!("users-{code}")
    } else if is_name_code(code) {
        format!("users-name-{code}")
    } else {
        format!("users-refused-{code}")
    };

    i18n::text_with(ctx, &id, &[("name", name)]).into_owned()
}

pub fn full_name_problem(ctx: &Context, code: &str) -> String {
    i18n::text(ctx, &format!("users-full-name-{code}")).into_owned()
}

/// What is wrong with a typed password, `mismatch` for two that differ.
pub fn password_problem(ctx: &Context, code: &str) -> String {
    i18n::text(ctx, &format!("users-password-{code}")).into_owned()
}

/// The note on why the lock flags are missing, `None` once they are known
/// or before the page asked.
pub fn locks(ctx: &Context, locks: &Locks) -> Option<String> {
    match locks {
        Locks::NotAsked | Locks::Known(_) => None,
        Locks::Loading => Some(i18n::text(ctx, "users-locks-loading").into_owned()),
        Locks::Refused(_) | Locks::Unknown => {
            let id = format!("users-locks-{}", locks.code());
            Some(format!(
                "{} {}",
                i18n::text(ctx, &id),
                i18n::text(ctx, "users-locks-again")
            ))
        }
    }
}

pub fn run(ctx: &Context, run: &PageRun) -> Option<String> {
    let state = run.state();
    if state.is_empty() {
        return None;
    }

    let id = format!("users-run-{}-{state}", run.kind());
    Some(i18n::text_with(ctx, &id, &[("user", run.user())]).into_owned())
}

pub fn list(ctx: &Context, names: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let separator = i18n::text(ctx, "users-list-separator");
    names
        .into_iter()
        .map(|name| name.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(&separator)
}

fn is_name_code(code: &str) -> bool {
    matches!(
        code,
        "empty" | "too-long" | "invalid-start" | "invalid-character" | "invalid"
    )
}
