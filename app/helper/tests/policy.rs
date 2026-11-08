//! The polkit policy in `data/` must match how the app starts the helper:
//! its installed path and the first argument of each command.

use std::error::Error;
use std::fs;
use std::path::Path;

const POLICY: &str = "../../data/org.garudalinux.toolbox.policy";
const HELPER: &str = "/usr/lib/garuda-toolbox/garuda-toolbox-helper";

type TestResult = Result<(), Box<dyn Error>>;

fn policy() -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(POLICY),
    )?)
}

fn action(policy: &str, id: &str) -> Result<String, Box<dyn Error>> {
    let (_, rest) = policy
        .split_once(&format!("<action id=\"{id}\">"))
        .ok_or_else(|| format!("no action {id}"))?;

    let (body, _) = rest
        .split_once("</action>")
        .ok_or_else(|| format!("action {id} is not closed"))?;

    Ok(body.to_owned())
}

fn annotation(key: &str, value: &str) -> String {
    format!("<annotate key=\"org.freedesktop.policykit.exec.{key}\">{value}</annotate>")
}

#[test]
fn apply_asks_for_an_admin_and_keeps_the_answer_for_a_while() -> TestResult {
    let apply = action(&policy()?, "org.garudalinux.toolbox.apply")?;

    assert!(apply.contains(&annotation("path", HELPER)));
    assert!(apply.contains(&annotation("argv1", "apply")));
    assert!(apply.contains("<allow_active>auth_admin_keep</allow_active>"));
    assert!(apply.contains("<allow_inactive>auth_admin</allow_inactive>"));
    Ok(())
}

#[test]
fn only_the_read_command_runs_without_a_password_in_an_active_session() -> TestResult {
    let policy = policy()?;
    let read = action(&policy, "org.garudalinux.toolbox.read")?;

    assert!(read.contains(&annotation("path", HELPER)));
    assert!(read.contains(&annotation("argv1", "read")));
    assert!(read.contains("<allow_active>yes</allow_active>"));
    assert!(read.contains("<allow_any>auth_admin</allow_any>"));
    assert_eq!(
        policy.matches("<allow_active>yes</allow_active>").count(),
        1,
        "no other action may run without a password"
    );
    assert_eq!(policy.matches("<action id=").count(), 4);
    Ok(())
}

/// The kernel log and the journal can hold what `kernel.dmesg_restrict`
/// and the journal's groups protect, so they never run without an admin.
#[test]
fn logs_ask_for_an_admin() -> TestResult {
    let logs = action(&policy()?, "org.garudalinux.toolbox.logs")?;

    assert!(logs.contains(&annotation("path", HELPER)));
    assert!(logs.contains(&annotation("argv1", "logs")));
    assert!(logs.contains("<allow_active>auth_admin_keep</allow_active>"));
    assert!(logs.contains("<allow_any>auth_admin</allow_any>"));
    Ok(())
}

/// A config file may hold what only root may read (`/etc/sudoers`, the
/// GRUB config), so handing one out never runs without an admin.
#[test]
fn fetching_a_config_file_asks_for_an_admin() -> TestResult {
    let fetch = action(&policy()?, "org.garudalinux.toolbox.fetch")?;

    assert!(fetch.contains(&annotation("path", HELPER)));
    assert!(fetch.contains(&annotation("argv1", "fetch")));
    assert!(fetch.contains("<allow_active>auth_admin_keep</allow_active>"));
    assert!(fetch.contains("<allow_inactive>auth_admin</allow_inactive>"));
    assert!(fetch.contains("<allow_any>auth_admin</allow_any>"));
    Ok(())
}
