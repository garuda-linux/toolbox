//! Home folders and user ids in the Arch test image: real `useradd -d`,
//! `-M`, `-u` and `-r`, and `usermod -d` with and without `-m`, through
//! the helper. Accounts made here are named `toolbox-test-user-*`, their
//! other homes live below `/srv/toolbox-test-homes`, and both are removed
//! again.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{
    FullName, HomeChange, HomeDirectory, NewHome, NewUser, NewUserParts, RootAction, UserId,
};

use super::common::{TestResult, helper, outcomes, request};
use super::users::{forget, new_user, passwd_entry, user};
use super::{apply, in_test_image, printed};

const HOMES: &str = "/srv/toolbox-test-homes";

fn placed(
    name: &str,
    home: NewHome,
    uid: Option<u32>,
    system: bool,
) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::CreateUser {
        user: NewUser::new(NewUserParts {
            name: user(name)?,
            full_name: FullName::default(),
            shell: None,
            groups: BTreeSet::new(),
            home,
            uid: uid.map(UserId::new).transpose()?,
            system,
        })?,
    })
}

fn at(folder: &str) -> Result<HomeDirectory, Box<dyn Error>> {
    Ok(HomeDirectory::new(format!("{HOMES}/{folder}"))?)
}

fn moved(name: &str, folder: &str, move_files: bool) -> Result<RootAction, Box<dyn Error>> {
    Ok(RootAction::SetHomeFolder {
        user: user(name)?,
        change: HomeChange {
            home: at(folder)?,
            move_files,
        },
    })
}

fn fields(name: &str) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(passwd_entry(name)?.split(':').map(str::to_owned).collect())
}

fn uid_of(name: &str) -> Result<u32, Box<dyn Error>> {
    Ok(fields(name)?
        .get(2)
        .ok_or("no user id in the entry")?
        .parse()?)
}

fn home_of(name: &str) -> Result<String, Box<dyn Error>> {
    Ok(fields(name)?
        .get(5)
        .ok_or("no home folder in the entry")?
        .clone())
}

/// An empty folder for the homes of one test, whatever an earlier run
/// left behind.
fn fresh_homes() -> TestResult {
    // A missing folder is the wanted state already.
    fs::remove_dir_all(HOMES).ok();
    fs::create_dir_all(HOMES)?;
    Ok(())
}

#[test]
fn a_user_gets_the_asked_home_folder_and_user_id() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-placed";
    forget(&[name], &[]);
    fresh_homes()?;

    assert_eq!(
        apply(vec![placed(
            name,
            NewHome::At(at("placed")?),
            Some(4711),
            false
        )?])?,
        [Outcome::Succeeded]
    );

    let home = Path::new(HOMES).join("placed");

    assert_eq!(uid_of(name)?, 4711);
    assert_eq!(home_of(name)?, format!("{HOMES}/placed"));
    assert!(home.is_dir());
    assert_eq!(fs::metadata(&home)?.uid(), 4711);
    assert!(!Path::new("/home").join(name).exists());

    forget(&[name], &[]);
    fs::remove_dir_all(HOMES).ok();
    Ok(())
}

#[test]
fn a_user_without_a_home_folder_gets_none_and_a_system_account_a_system_id() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let homeless = "toolbox-test-user-homeless";
    let service = "toolbox-test-user-service";
    forget(&[homeless, service], &[]);

    assert_eq!(
        apply(vec![
            placed(homeless, NewHome::Absent, None, false)?,
            placed(service, NewHome::Absent, None, true)?,
        ])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );

    assert!(!Path::new("/home").join(homeless).exists());
    assert!(!Path::new("/home").join(service).exists());
    assert!(uid_of(homeless)? >= 1000, "{}", passwd_entry(homeless)?);
    assert!(uid_of(service)? < 1000, "{}", passwd_entry(service)?);

    forget(&[homeless, service], &[]);
    Ok(())
}

#[test]
fn useradd_refuses_a_user_id_that_is_taken() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let first = "toolbox-test-user-first";
    let second = "toolbox-test-user-second";
    forget(&[first, second], &[]);

    assert_eq!(
        apply(vec![placed(first, NewHome::Absent, Some(4712), false)?])?,
        [Outcome::Succeeded]
    );

    let run = helper(
        &["apply"],
        &request(
            Mode::Run,
            vec![placed(second, NewHome::Absent, Some(4712), false)?],
        )?,
    )?;

    assert!(
        matches!(
            outcomes(&run)?.as_slice(),
            [Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }]
        ),
        "{}",
        run.stdout
    );
    assert!(printed(&run)?.contains("4712"), "{}", printed(&run)?);
    assert_eq!(passwd_entry(second)?, "");

    forget(&[first, second], &[]);
    Ok(())
}

#[test]
fn a_home_folder_moves_with_its_files_or_only_in_the_entry() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-mover";
    forget(&[name], &[]);
    fresh_homes()?;

    assert_eq!(apply(vec![new_user(name, &[])?])?, [Outcome::Succeeded]);

    let old = Path::new("/home").join(name);

    fs::write(old.join("note.txt"), "kept\n")?;

    assert_eq!(
        apply(vec![moved(name, "moved", true)?])?,
        [Outcome::Succeeded]
    );

    let new = Path::new(HOMES).join("moved");

    assert_eq!(home_of(name)?, format!("{HOMES}/moved"));
    assert_eq!(fs::read_to_string(new.join("note.txt"))?, "kept\n");
    assert!(!old.exists(), "the old home folder is still there");

    assert_eq!(
        apply(vec![moved(name, "entry-only", false)?])?,
        [Outcome::Succeeded]
    );

    assert_eq!(home_of(name)?, format!("{HOMES}/entry-only"));
    assert!(new.join("note.txt").is_file(), "the files moved without -m");
    assert!(!Path::new(HOMES).join("entry-only").exists());

    forget(&[name], &[]);
    fs::remove_dir_all(HOMES).ok();
    Ok(())
}

#[test]
fn a_move_onto_an_existing_folder_fails_and_changes_nothing() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let name = "toolbox-test-user-blocked";
    forget(&[name], &[]);
    fresh_homes()?;
    fs::create_dir_all(Path::new(HOMES).join("taken"))?;

    assert_eq!(apply(vec![new_user(name, &[])?])?, [Outcome::Succeeded]);

    let run = helper(
        &["apply"],
        &request(Mode::Run, vec![moved(name, "taken", true)?])?,
    )?;

    assert!(
        matches!(
            outcomes(&run)?.as_slice(),
            [Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }]
        ),
        "{}",
        run.stdout
    );
    assert_eq!(home_of(name)?, format!("/home/{name}"));
    assert!(Path::new("/home").join(name).is_dir());

    forget(&[name], &[]);
    fs::remove_dir_all(HOMES).ok();
    Ok(())
}

#[test]
fn the_home_folder_of_root_stays() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let before = passwd_entry("root")?;
    let run = helper(
        &["apply"],
        &request(Mode::Run, vec![moved("root", "root", true)?])?,
    )?;

    assert!(
        matches!(
            outcomes(&run)?.as_slice(),
            [Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }]
        ),
        "{}",
        run.stdout
    );
    assert!(printed(&run)?.contains("the user id 0, outside 1000 to 60000"));
    assert_eq!(passwd_entry("root")?, before);
    assert!(Path::new("/root").is_dir());
    Ok(())
}
