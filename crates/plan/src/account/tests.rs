use super::{AccountLock, FullName, HomeFolder, InvalidFullName, MAX_FULL_NAME_BYTES, ShadowLocks};
use crate::UserName;

/// `/etc/shadow` of the Lima image with every salt and hash replaced by
/// `x`, the `$` fields and the lock sign kept.
pub(super) const SHADOW: &str = crate::testing::account::SHADOW;
/// `/etc/passwd` of the Lima image, its user renamed to `user`.
pub(super) const PASSWD: &str = crate::testing::accounts::PASSWD;

/// Edit: the locked `guest` hash unlocked, plus a line with a name no
/// account has and a line with no fields.
fn edited() -> String {
    let unlocked = SHADOW.replace("guest:!$y$", "guest:$y$");

    format!("{unlocked}Bad Name:!:1::::::\nbroken\n")
}

fn lock(user: &str, locked: bool) -> AccountLock {
    AccountLock {
        user: UserName::new(user).unwrap(),
        locked,
    }
}

#[test]
fn every_full_name_of_a_real_passwd_passes() {
    for gecos in PASSWD.lines().map(|line| line.split(':').nth(4).unwrap()) {
        assert_eq!(FullName::new(gecos).unwrap().as_str(), gecos);
    }
}

/// Names a user types, for the scripts no captured passwd has.
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
fn a_full_name_hides_no_character() {
    // The `user` of the captured passwd, given a GECOS name.
    let line = PASSWD
        .lines()
        .find(|line| line.starts_with("user:"))
        .unwrap();

    let named = line.replacen("::", ":Ada Lovelace:", 1);
    let gecos = named.split(':').nth(4).unwrap();
    assert_eq!(FullName::new(gecos).unwrap().as_str(), "Ada Lovelace");

    for hidden in ['\u{202e}', '\u{200d}'] {
        let spoofed = gecos.replacen(' ', &format!(" {hidden}"), 1);

        assert_eq!(
            FullName::new(spoofed),
            Err(InvalidFullName::InvalidCharacter(hidden))
        );
    }
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
fn shadow_locks_follow_the_exclamation_mark_and_the_star() {
    let locks = ShadowLocks::from_shadow(SHADOW).locks;
    let unlocked: Vec<&str> = locks
        .iter()
        .filter(|lock| !lock.locked)
        .map(|lock| lock.user.as_str())
        .collect();

    assert_eq!(locks.len(), 27);
    assert_eq!(locks[0], lock("root", true));
    assert!(locks.contains(&lock("nobody", true)));
    assert!(locks.contains(&lock("user", true)));
    assert!(locks.contains(&lock("guest", true)));
    assert_eq!(locks[26], lock("nopass", false));
    assert_eq!(unlocked, ["nopass"]);
}

#[test]
fn an_unlocked_hash_and_lines_without_an_account_are_read_right() {
    let locks = ShadowLocks::from_shadow(&edited()).locks;

    assert_eq!(locks.len(), 27);
    assert!(locks.contains(&lock("guest", false)));
}

#[test]
fn no_part_of_a_hash_leaves_the_parser() {
    let shadow = edited();
    let answer = serde_json::to_string(&ShadowLocks::from_shadow(&shadow)).unwrap();

    for secret in ["$y$", "j9T", "xxxx", "20765", "14871"] {
        assert!(shadow.contains(secret), "{secret} not in the fixture");
        assert!(!answer.contains(secret), "{secret} in {answer}");
    }

    assert_eq!(
        serde_json::from_str::<ShadowLocks>(&answer).unwrap(),
        ShadowLocks::from_shadow(&shadow)
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
