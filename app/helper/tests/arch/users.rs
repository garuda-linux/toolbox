//! Accounts in the Arch test image: real `useradd` and `userdel` through
//! the helper, and the helpers the password, group and home tests share.
//! Every account made here is named `toolbox-test-user-*` and removed
//! again, also after a failed assertion of an earlier run.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::Path;
use std::process::Command;

use toolbox_plan::protocol::{Mode, Outcome, decode};
use toolbox_plan::{
    FullName, GroupName, HomeFolder, NewHome, NewUser, NewUserParts, Password, RootAction,
    ShadowLocks, ShellPath, StepId, UserName,
};

use super::common::{Run, TestResult, helper, outcomes, request};
use super::{apply, in_test_image, printed};

pub const SECRET: &str = "Toolbox test: pass phrase 4711";
pub const SECOND_SECRET: &str = "another-Secret:with:colons";

pub fn user(name: &str) -> Result<UserName, Box<dyn Error>> {
    Ok(UserName::new(name)?)
}

pub fn group(name: &str) -> Result<GroupName, Box<dyn Error>> {
    Ok(GroupName::new(name)?)
}

fn output_of(program: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new(program).args(args).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn passwd_entry(name: &str) -> Result<String, Box<dyn Error>> {
    output_of("getent", &["passwd", name])
}

pub fn group_entry(name: &str) -> Result<String, Box<dyn Error>> {
    output_of("getent", &["group", name])
}

/// The second word of `passwd -S`: `P` usable password, `L` locked, `NP`
/// none.
pub fn password_state(name: &str) -> Result<String, Box<dyn Error>> {
    Ok(output_of("passwd", &["-S", name])?
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned())
}

/// Leaves no trace of an account, whatever an earlier test left behind.
pub fn forget(users: &[&str], groups: &[&str]) {
    for name in users {
        // A missing account is the wanted state already.
        Command::new("userdel")
            .args(["-r", "--", name])
            .output()
            .ok();
        std::fs::remove_dir_all(Path::new("/home").join(name)).ok();
    }

    for name in groups {
        Command::new("groupdel").args(["--", name]).output().ok();
    }
}

pub fn new_user(name: &str, groups: &[&str]) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::CreateUser {
        user: NewUser::new(NewUserParts {
            name: user(name)?,
            full_name: FullName::new("Toolbox Test")?,
            shell: Some(ShellPath::new("/usr/bin/bash")?),
            groups: groups
                .iter()
                .map(|member_of| group(member_of))
                .collect::<Result<BTreeSet<_>, _>>()?,
            home: NewHome::Standard,
            uid: None,
            system: false,
        })?,
    })
}

pub fn set_password(name: &str) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::SetPassword {
        user: user(name)?,
        password: None,
    })
}

pub fn run_with_secrets(
    actions: Vec<RootAction>,
    secrets: &[(u32, &str)],
) -> Result<Run, Box<dyn Error>> {
    let mut input = request(Mode::Run, actions)?;

    for (step, secret) in secrets {
        let line = Password::new((*secret).to_owned())?.wire_line(StepId::new(*step));

        input.push_str(std::str::from_utf8(line.expose())?);
    }

    helper(&["apply"], &input)
}

pub fn can_log_in(name: &str, password: &str) -> Result<bool, Box<dyn Error>> {
    use std::io::Write as _;
    use std::process::Stdio;

    let mut child = Command::new("su")
        .args(["tester", "-c"])
        .arg(format!("su {name} -c true"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "{password}")?;
    }

    Ok(child.wait()?.success())
}

fn locks() -> Result<ShadowLocks, Box<dyn Error>> {
    let output = crate::oneshot::output(&["read", "shadow-locks"])?;

    assert!(output.status.success());
    Ok(decode(String::from_utf8(output.stdout)?.trim_end())?)
}

pub fn locked(name: &str) -> Result<Option<bool>, Box<dyn Error>> {
    Ok(locks()?
        .locks
        .into_iter()
        .find(|lock| lock.user.as_str() == name)
        .map(|lock| lock.locked))
}

#[test]
fn a_user_is_added_with_a_password_that_never_shows() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-add";
    forget(&[name], &[]);

    let run = run_with_secrets(
        vec![
            new_user(name, &["toolbox-test-group"])?,
            set_password(name)?,
        ],
        &[(2, SECRET)],
    )?;

    assert_eq!(
        outcomes(&run)?,
        [Outcome::Succeeded, Outcome::Succeeded],
        "{}\n{}",
        run.stderr,
        printed(&run)?
    );

    let entry = passwd_entry(name)?;

    assert!(entry.contains(":Toolbox Test:"), "{entry}");
    assert!(
        entry.ends_with(&format!(":/home/{name}:/usr/bin/bash")),
        "{entry}"
    );
    assert!(Path::new("/home").join(name).is_dir());
    assert!(group_entry("toolbox-test-group")?.contains(name));
    assert_eq!(password_state(name)?, "P");
    assert!(can_log_in(name, SECRET)?);
    assert!(!can_log_in(name, "not the password")?);

    for text in [&run.stdout, &run.stderr] {
        assert!(!text.contains(SECRET), "the secret shows in {text}");
        assert!(!text.contains("pass phrase"), "part of the secret shows");
    }

    forget(&[name], &[]);
    Ok(())
}

#[test]
fn the_root_account_is_neither_locked_nor_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let before = passwd_entry("root")?;
    let locked_before = locked("root")?;

    for action in [
        RootAction::SetUserLocked {
            user: user("root")?,
            locked: true,
        },
        RootAction::DeleteUser {
            user: user("root")?,
            home: HomeFolder::Delete,
        },
    ] {
        let run = helper(&["apply"], &request(Mode::Run, vec![action])?)?;

        assert!(
            matches!(outcomes(&run)?.as_slice(), [Outcome::Failed { cause: _ }]),
            "{}",
            run.stdout
        );
        assert!(printed(&run)?.contains("the user id 0, outside 1000 to 60000"));
    }

    assert_eq!(passwd_entry("root")?, before);
    assert_eq!(locked("root")?, locked_before);
    assert!(Path::new("/root").is_dir());
    Ok(())
}

#[test]
fn a_user_is_removed_with_or_without_the_home() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let kept = "toolbox-test-user-keep";
    let gone = "toolbox-test-user-gone";
    forget(&[kept, gone], &[]);

    assert_eq!(
        apply(vec![new_user(kept, &[])?, new_user(gone, &[])?])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );
    assert!(Path::new("/home").join(kept).is_dir() && Path::new("/home").join(gone).is_dir());

    assert_eq!(
        apply(vec![
            RootAction::DeleteUser {
                user: user(kept)?,
                home: HomeFolder::Keep
            },
            RootAction::DeleteUser {
                user: user(gone)?,
                home: HomeFolder::Delete
            },
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );

    assert_eq!(passwd_entry(kept)?, "");
    assert_eq!(passwd_entry(gone)?, "");
    assert!(Path::new("/home").join(kept).is_dir());
    assert!(!Path::new("/home").join(gone).exists());

    forget(&[kept, gone], &[]);
    Ok(())
}
