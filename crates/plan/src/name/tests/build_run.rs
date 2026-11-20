use crate::BuildRun;
use crate::name::{InvalidName, NameKind, Reason};

fn refused(value: &str, reason: Reason) {
    assert_eq!(
        BuildRun::new(value).unwrap_err(),
        InvalidName::new(NameKind::BuildRun, value, reason)
    );
}

#[test]
fn a_build_run_is_lowercase_letters_and_digits() {
    assert_eq!(
        BuildRun::new("20261119t221500a").unwrap().as_str(),
        "20261119t221500a"
    );
}

/// The helper joins it to a folder below the user's home, so nothing in
/// it may leave that folder or read as an option.
#[test]
fn a_build_run_never_names_another_folder() {
    refused("", Reason::Empty);
    refused("../etc", Reason::InvalidStart('.'));
    refused("a/b", Reason::InvalidCharacter('/'));
    refused("-r", Reason::InvalidStart('-'));
    refused("rUn1", Reason::InvalidCharacter('U'));
    refused(&"a".repeat(33), Reason::TooLong { max: 32 });
}
