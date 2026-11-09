//! A home below any folder everyone may write is refused, however far up
//! that folder is, for a move and for a new account alike.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;

use toolbox_plan::HomeDirectory;
use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::Outcome;

use crate::testing::{assert_failed, run_builtin as run};

fn home() -> HomeDirectory {
    HomeDirectory::new("/srv/shared/people/ada").unwrap()
}

#[test]
fn a_shared_folder_two_levels_up_refuses_the_move() {
    let dir = tempfile::tempdir().unwrap();
    let shared = dir.path().join("srv/shared");

    fs::create_dir_all(shared.join("people")).unwrap();
    fs::set_permissions(&shared, fs::Permissions::from_mode(0o1777)).unwrap();

    let (outcome, text) = run(dir.path(), &Builtin::RequireFreeHome(home()));

    assert_failed(outcome);
    assert!(text.contains("writable for everyone"), "{text}");
}

/// The parents useradd will make do not exist yet. The nearest one that
/// does decides.
#[test]
fn a_new_account_s_home_is_checked_up_to_the_nearest_folder_that_exists() {
    let dir = tempfile::tempdir().unwrap();
    let srv = dir.path().join("srv");

    fs::create_dir_all(&srv).unwrap();

    let safe = Builtin::RequireSafeHome(home());

    assert_eq!(run(dir.path(), &safe).0, Outcome::Succeeded);

    fs::set_permissions(&srv, fs::Permissions::from_mode(0o777)).unwrap();

    let (outcome, text) = run(dir.path(), &safe);

    assert_failed(outcome);
    assert!(text.contains("writable for everyone"), "{text}");
}

/// A new account's home may exist already, useradd then only takes it.
#[test]
fn a_new_account_s_existing_home_is_no_reason_to_refuse() {
    let dir = tempfile::tempdir().unwrap();

    fs::create_dir_all(dir.path().join("srv/shared/people/ada")).unwrap();

    assert_eq!(
        run(dir.path(), &Builtin::RequireSafeHome(home())).0,
        Outcome::Succeeded
    );
}
