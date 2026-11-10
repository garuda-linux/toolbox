use proptest::prelude::*;

use super::{KnownPackage, is_package_name};
use crate::PackageName;
use crate::name::rules::MAX_PACKAGE_CHARS;

const MELD: KnownPackage = KnownPackage::new("meld");

#[test]
fn a_known_package_becomes_the_same_package_name() {
    assert_eq!(MELD.name(), PackageName::new("meld").unwrap());
}

#[test]
fn the_compile_time_check_refuses_what_the_rule_refuses() {
    let too_long = "a".repeat(MAX_PACKAGE_CHARS + 1);

    for value in ["", "-rf", ".hidden", "a b", "a;b", "ä", too_long.as_str()] {
        assert!(!is_package_name(value), "{value:?}");
        assert!(PackageName::new(value).is_err(), "{value:?}");
    }
}

proptest! {
    #[test]
    fn the_compile_time_check_agrees_with_the_rule(value in "\\PC{0,8}|[@_+a-z0-9.-]{0,12}") {
        prop_assert_eq!(is_package_name(&value), PackageName::new(&value).is_ok());
    }
}
