//! systemd's sub states as a small set of codes, because the page names
//! each one in words and systemd adds new ones over time.

#[cfg(test)]
mod tests;

use super::ActiveState;

/// `""` where it adds nothing to the active state, `"other"` for a state newer than this app.
pub fn sub_code(active: ActiveState, sub: &str) -> &'static str {
    let code = fold(sub);
    let repeated = match active {
        ActiveState::Active => matches!(code, "running" | "active"),
        ActiveState::Inactive => code == "dead",
        ActiveState::Failed => code == "failed",
        ActiveState::Activating => code == "starting",
        ActiveState::Deactivating => code == "stopping",
        ActiveState::Reloading => code == "reloading",
        ActiveState::Maintenance | ActiveState::Refreshing | ActiveState::Other => false,
    };

    if repeated { "" } else { code }
}

/// The steps of one change (`start-pre`, `start`, `start-post`) are one
/// code, because they pass too fast to read and say the same to a user.
fn fold(sub: &str) -> &'static str {
    match sub {
        "" => "",
        "running" => "running",
        "exited" => "exited",
        "listening" => "listening",
        "waiting" => "waiting",
        "active" => "active",
        "mounted" => "mounted",
        "plugged" => "plugged",
        "elapsed" => "elapsed",
        "abandoned" => "abandoned",
        "failed" => "failed",
        "cleaning" => "cleaning",
        "dead" | "dead-resources-pinned" => "dead",
        "auto-restart"
        | "auto-restart-queued"
        | "dead-before-auto-restart"
        | "failed-before-auto-restart" => "restarting",
        "condition" => "starting",
        _ if has_prefix(sub, &["start", "mounting", "activating"]) => "starting",
        _ if has_prefix(sub, &["stop", "final", "unmounting", "deactivating"]) => "stopping",
        _ if has_prefix(sub, &["reload", "remounting"]) => "reloading",
        _ => "other",
    }
}

/// `stop-sigterm` belongs to `stop`, `stopped` would not.
fn has_prefix(sub: &str, stems: &[&str]) -> bool {
    stems.iter().any(|stem| {
        sub.strip_prefix(stem)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
    })
}
