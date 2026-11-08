use super::{AccountLock, FullName, HomeFolder, InvalidFullName, MAX_FULL_NAME_BYTES, ShadowLocks};
use crate::UserName;

const SHADOW: &str = "root:$6$rounds=5000$saltsalt$hashhashhash:19000::::::
bin:!*:19000::::::
nico:$y$j9T$abcdefgh$0123456789abcdef:19500:0:99999:7:::
locked:!$y$j9T$abcdefgh$fedcba9876543210:19500:0:99999:7:::
fresh:!:19500:0:99999:7:::
nopass::19500:0:99999:7:::
Bad Name:!:1::::::
broken
";

fn lock(user: &str, locked: bool) -> AccountLock {
    AccountLock {
        user: UserName::new(user).unwrap(),
        locked,
    }
}

#[test]
fn full_names_take_ordinary_names_and_the_empty_one() {
    for name in [
        "",
        "Nico Jensch",
        "José Ñandú",
        "王小明",
        "O'Brien (work)",
        "a-b",
    ] {
        assert_eq!(FullName::new(name).unwrap().as_str(), name);
    }

    assert!(FullName::default().is_empty());
}

#[test]
fn full_names_refuse_what_would_break_passwd_or_gecos() {
    for (text, character) in [
        ("a:b", ':'),
        ("a,b", ','),
        ("a=b", '='),
        ("a\"b", '"'),
        ("a\nroot::0:0", '\n'),
        ("a\tb", '\t'),
        ("a\0b", '\0'),
    ] {
        assert_eq!(
            FullName::new(text),
            Err(InvalidFullName::InvalidCharacter(character)),
            "{}",
            text.escape_debug()
        );
    }

    assert_eq!(FullName::new("-c"), Err(InvalidFullName::LeadingDash));
    assert_eq!(FullName::new(" Nico"), Err(InvalidFullName::OuterBlank));
    assert_eq!(FullName::new("Nico "), Err(InvalidFullName::OuterBlank));
    assert_eq!(
        FullName::new("x".repeat(MAX_FULL_NAME_BYTES + 1)),
        Err(InvalidFullName::TooLong {
            max: MAX_FULL_NAME_BYTES
        })
    );
}

#[test]
fn full_names_are_checked_when_decoded() {
    assert_eq!(
        serde_json::from_str::<FullName>("\"Nico Jensch\"").unwrap(),
        FullName::new("Nico Jensch").unwrap()
    );
    serde_json::from_str::<FullName>("\"a:b\"").unwrap_err();
    assert_eq!(
        serde_json::to_string(&FullName::new("Nico").unwrap()).unwrap(),
        "\"Nico\""
    );
}

#[test]
fn full_name_codes_are_stable() {
    assert_eq!(
        FullName::new("a:b").unwrap_err().as_str(),
        "invalid-character"
    );
    assert_eq!(FullName::new("-a").unwrap_err().as_str(), "leading-dash");
    assert_eq!(FullName::new("a ").unwrap_err().as_str(), "outer-blank");
    assert_eq!(
        FullName::new("x".repeat(200)).unwrap_err().as_str(),
        "too-long"
    );
}

#[test]
fn the_home_folder_stays_by_default() {
    assert_eq!(HomeFolder::default(), HomeFolder::Keep);
    assert_eq!(
        serde_json::to_string(&HomeFolder::Delete).unwrap(),
        "\"delete\""
    );
}

#[test]
fn shadow_locks_follow_the_exclamation_mark() {
    assert_eq!(
        ShadowLocks::from_shadow(SHADOW).locks,
        [
            lock("root", false),
            lock("bin", true),
            lock("nico", false),
            lock("locked", true),
            lock("fresh", true),
            lock("nopass", false),
        ]
    );
}

#[test]
fn no_part_of_a_hash_leaves_the_parser() {
    let answer = serde_json::to_string(&ShadowLocks::from_shadow(SHADOW)).unwrap();

    for secret in [
        "$6$",
        "$y$",
        "saltsalt",
        "hashhash",
        "abcdefgh",
        "0123456789",
        "19500",
    ] {
        assert!(!answer.contains(secret), "{secret} in {answer}");
    }

    assert_eq!(
        serde_json::from_str::<ShadowLocks>(&answer).unwrap(),
        ShadowLocks::from_shadow(SHADOW)
    );
}

#[test]
fn an_empty_shadow_has_no_locks() {
    assert_eq!(ShadowLocks::from_shadow(""), ShadowLocks::default());
}

#[test]
fn a_full_name_parses_from_str_and_borrows_as_str() {
    let parsed: FullName = "Ada Lovelace".parse().unwrap();

    assert_eq!(parsed.as_ref(), "Ada Lovelace");
    assert_eq!(parsed.to_string(), "Ada Lovelace");
    "a:b".parse::<FullName>().unwrap_err();
}
