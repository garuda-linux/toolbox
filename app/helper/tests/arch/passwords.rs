//! Passwords and locks in the Arch test image: real `chpasswd` and
//! `usermod` through the helper with the secret lines the app sends, and
//! the `read shadow-locks` answer. Accounts are made with the helpers of
//! the users module and removed again.

use std::error::Error;

use toolbox_plan::RootAction;
use toolbox_plan::protocol::{Event, Mode, Outcome};

use super::common::{TestResult, events, helper, outcomes, request};
use super::users::{
    SECOND_SECRET, SECRET, can_log_in, forget, locked, new_user, password_state, run_with_secrets,
    set_password, user,
};
use super::{apply, in_test_image};

#[test]
fn a_password_is_changed_and_a_missing_secret_changes_nothing() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-pw";
    forget(&[name], &[]);
    assert_eq!(apply(vec![new_user(name, &[])?])?, [Outcome::Succeeded]);
    assert_eq!(password_state(name)?, "L");

    let run = run_with_secrets(vec![set_password(name)?], &[(1, SECOND_SECRET)])?;

    assert_eq!(outcomes(&run)?, [Outcome::Succeeded], "{}", run.stderr);
    assert!(can_log_in(name, SECOND_SECRET)?);
    assert!(!run.stdout.contains(SECOND_SECRET) && !run.stderr.contains(SECOND_SECRET));

    // A line for another step, and an answer where the secret belongs:
    // both reject the whole request.
    for line in [
        format!("secret 7 {SECRET}\n"),
        "{\"type\":\"cancel\"}\n".to_owned(),
    ] {
        let input = format!("{}{line}", request(Mode::Run, vec![set_password(name)?])?);
        let rejected = helper(&["apply"], &input)?;

        assert!(matches!(
            events(&rejected)?.first(),
            Some(Event::Rejected { reason: _ })
        ));
        assert!(!rejected.stdout.contains(SECRET) && !rejected.stderr.contains(SECRET));
    }

    assert!(can_log_in(name, SECOND_SECRET)?);
    forget(&[name], &[]);
    Ok(())
}

#[test]
fn an_account_is_locked_and_unlocked() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-lock";
    forget(&[name], &[]);

    let made = run_with_secrets(
        vec![new_user(name, &[])?, set_password(name)?],
        &[(2, SECRET)],
    )?;

    assert_eq!(outcomes(&made)?, [Outcome::Succeeded, Outcome::Succeeded]);
    assert_eq!(locked(name)?, Some(false));

    let lock = |locked: bool| -> Result<RootAction, Box<dyn Error>> {
        Ok(RootAction::SetUserLocked {
            user: user(name)?,
            locked,
        })
    };

    assert_eq!(apply(vec![lock(true)?])?, [Outcome::Succeeded]);
    assert_eq!(password_state(name)?, "L");
    assert_eq!(locked(name)?, Some(true));
    assert!(!can_log_in(name, SECRET)?);

    assert_eq!(apply(vec![lock(false)?])?, [Outcome::Succeeded]);
    assert_eq!(password_state(name)?, "P");
    assert_eq!(locked(name)?, Some(false));
    assert!(can_log_in(name, SECRET)?);

    forget(&[name], &[]);
    Ok(())
}

#[test]
fn shadow_locks_answers_flags_and_no_hash() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let output = crate::oneshot::output(&["read", "shadow-locks"])?;

    let answer = String::from_utf8(output.stdout)?;
    let shadow = std::fs::read_to_string("/etc/shadow")?;

    assert!(output.status.success());
    assert_eq!(answer.lines().count(), 1);
    assert!(locked("root")?.is_some() && locked("tester")?.is_some());

    for hash in shadow
        .lines()
        .filter_map(|line| line.split(':').nth(1))
        .filter(|field| field.len() > 2)
    {
        assert!(!answer.contains(hash.trim_start_matches('!')));
    }

    Ok(())
}

#[test]
fn a_misplaced_secret_line_rejects_the_whole_request() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let input = format!(
        "{}not-a-secret-line\n",
        request(Mode::Run, vec![set_password("toolbox-test-user-add")?])?
    );

    let run = helper(&["apply"], &input)?;
    let first = events(&run)?.into_iter().next();

    assert!(!run.status.success());
    assert!(
        matches!(&first, Some(Event::Rejected { reason }) if reason.contains("secret")),
        "{first:?}"
    );
    assert!(
        !run.stdout.contains("not-a-secret-line"),
        "the line is quoted in {}",
        run.stdout
    );
    Ok(())
}
