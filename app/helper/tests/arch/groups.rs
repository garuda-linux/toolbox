//! Groups and full names in the Arch test image: real `groupadd`,
//! `groupdel`, `gpasswd` and `chfn` through the helper. Accounts made here
//! are named `toolbox-test-user-*` or `toolbox-test-grp-*` and removed
//! again.

use std::process::Command;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{FullName, RootAction};

use super::common::{TestResult, helper, outcomes, request};
use super::users::{forget, group, group_entry, new_user, passwd_entry, user};
use super::{apply, in_test_image, printed};

#[test]
fn groups_are_added_joined_left_and_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-member";
    let makers = "toolbox-test-grp-makers";
    forget(&[name], &[makers]);

    assert_eq!(
        apply(vec![
            RootAction::CreateGroup {
                group: group(makers)?
            },
            new_user(name, &[])?,
            RootAction::AddToGroup {
                user: user(name)?,
                group: group(makers)?
            },
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded, Outcome::Succeeded]
    );

    let entry = group_entry(makers)?;
    let gid: u32 = entry.split(':').nth(2).unwrap_or_default().parse()?;

    assert!(gid >= 1000, "{entry}");
    assert!(entry.ends_with(&format!(":{name}")), "{entry}");

    assert_eq!(
        apply(vec![
            RootAction::RemoveFromGroup {
                user: user(name)?,
                group: group(makers)?
            },
            RootAction::DeleteGroup {
                group: group(makers)?
            },
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );
    assert_eq!(group_entry(makers)?, "");

    forget(&[name], &[makers]);
    Ok(())
}

#[test]
fn a_system_group_is_not_removed() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let before = group_entry("wheel")?;
    let run = helper(
        &["apply"],
        &request(
            Mode::Run,
            vec![RootAction::DeleteGroup {
                group: group("wheel")?,
            }],
        )?,
    )?;

    assert!(matches!(
        outcomes(&run)?.as_slice(),
        [Outcome::Failed { cause: _ }]
    ));
    assert!(printed(&run)?.contains("system group"));
    assert_eq!(group_entry("wheel")?, before);
    Ok(())
}

#[test]
fn the_full_name_changes_and_other_gecos_parts_stay() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-name";
    forget(&[name], &[]);
    assert_eq!(apply(vec![new_user(name, &[])?])?, [Outcome::Succeeded]);
    assert!(
        Command::new("usermod")
            .args(["-c", "Toolbox Test,Room 1,555,556", "--", name])
            .status()?
            .success()
    );

    assert_eq!(
        apply(vec![RootAction::SetFullName {
            user: user(name)?,
            full_name: FullName::new("Renamed Tester")?
        }])?,
        [Outcome::Succeeded]
    );
    assert!(
        passwd_entry(name)?.contains(":Renamed Tester,Room 1,555,556:"),
        "{}",
        passwd_entry(name)?
    );

    forget(&[name], &[]);
    Ok(())
}
