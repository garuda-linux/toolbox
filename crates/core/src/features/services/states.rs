//! systemd's sub states as a small set of codes, because the page names
//! each one in words and systemd adds new ones over time.

#[cfg(test)]
mod tests;

use super::ActiveState;

/// A sub state the page names, `Other` for one newer than this app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubCode {
    Running,
    Exited,
    Listening,
    Waiting,
    Active,
    Mounted,
    Plugged,
    Elapsed,
    Abandoned,
    Failed,
    Cleaning,
    Dead,
    Restarting,
    Starting,
    Stopping,
    Reloading,
    Other,
}

/// `None` where the sub state adds nothing to the active state.
pub fn sub_code(active: ActiveState, sub: &str) -> Option<SubCode> {
    let code = fold(sub)?;
    let repeated = match active {
        ActiveState::Active => matches!(code, SubCode::Running | SubCode::Active),
        ActiveState::Inactive => code == SubCode::Dead,
        ActiveState::Failed => code == SubCode::Failed,
        ActiveState::Activating => code == SubCode::Starting,
        ActiveState::Deactivating => code == SubCode::Stopping,
        ActiveState::Reloading => code == SubCode::Reloading,
        ActiveState::Maintenance | ActiveState::Refreshing | ActiveState::Other => false,
    };

    (!repeated).then_some(code)
}

/// The steps of one change (`start-pre`, `start`, `start-post`) are one
/// code, because they pass too fast to read and say the same to a user.
fn fold(sub: &str) -> Option<SubCode> {
    let code = match sub {
        "" => return None,
        "running" => SubCode::Running,
        "exited" => SubCode::Exited,
        "listening" => SubCode::Listening,
        "waiting" => SubCode::Waiting,
        "active" => SubCode::Active,
        "mounted" => SubCode::Mounted,
        "plugged" => SubCode::Plugged,
        "elapsed" => SubCode::Elapsed,
        "abandoned" => SubCode::Abandoned,
        "failed" => SubCode::Failed,
        "cleaning" => SubCode::Cleaning,
        "dead" | "dead-resources-pinned" => SubCode::Dead,
        "auto-restart"
        | "auto-restart-queued"
        | "dead-before-auto-restart"
        | "failed-before-auto-restart" => SubCode::Restarting,
        "condition" => SubCode::Starting,
        _ if has_prefix(sub, &["start", "mounting", "activating"]) => SubCode::Starting,
        _ if has_prefix(sub, &["stop", "final", "unmounting", "deactivating"]) => SubCode::Stopping,
        _ if has_prefix(sub, &["reload", "remounting"]) => SubCode::Reloading,
        _ => SubCode::Other,
    };

    Some(code)
}

/// `stop-sigterm` belongs to `stop`, `stopped` would not.
fn has_prefix(sub: &str, stems: &[&str]) -> bool {
    stems.iter().any(|stem| {
        sub.strip_prefix(stem)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
    })
}
