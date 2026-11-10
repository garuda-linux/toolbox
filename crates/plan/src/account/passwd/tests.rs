use super::{PasswdEntry, passwd_entries};
use crate::lines::lossy;

/// `/etc/passwd` of the Lima image, its user renamed to `user`.
const PASSWD: &str = crate::testing::accounts::PASSWD;

#[test]
fn every_entry_of_a_real_passwd_parses() {
    assert_eq!(passwd_entries(PASSWD).count(), PASSWD.lines().count());
}

#[test]
fn the_fields_come_in_passwd_order() {
    let user = passwd_entries(PASSWD).find(|entry| entry.name == "user");

    assert_eq!(
        user,
        Some(PasswdEntry {
            name: "user",
            uid: 1000,
            gid: 1000,
            gecos: "",
            home: "/home/user.guest",
            shell: "/bin/bash",
        })
    );
    let nobody = u32::from(u16::MAX - 1);

    assert_eq!(
        passwd_entries(PASSWD)
            .find(|entry| entry.name == "nobody")
            .map(|entry| (entry.uid, entry.gid)),
        Some((nobody, nobody))
    );
}

/// Edit: the empty full name of `user` replaced by `Ren\xE9`, a Latin-1
/// byte, and the file read lossily as both callers read it.
#[test]
fn a_full_name_that_is_no_utf8_keeps_the_entry() {
    let (before, after) = PASSWD.split_once("user:x:1000:1000:").unwrap();
    let text = lossy(
        [
            before.as_bytes(),
            b"user:x:1000:1000:Ren\xE9",
            after.as_bytes(),
        ]
        .concat(),
    );

    let user = passwd_entries(&text).find(|entry| entry.name == "user");

    assert_eq!(user.map(|entry| entry.uid), Some(1000));
    assert_eq!(user.map(|entry| entry.gecos), Some("Ren\u{fffd}"));
}

/// Edit: the user id of `user` replaced by `abc` and the group id of
/// `guest` cut off, as a hand-edited file could have it.
#[test]
fn an_entry_whose_ids_are_no_numbers_is_left_out() {
    let edited = PASSWD
        .replace("user:x:1000:1000:", "user:x:abc:1000:")
        .replace(
            "guest:x:1001:1001::/home/guest:/usr/bin/bash",
            "guest:x:1001",
        );

    let names: Vec<&str> = passwd_entries(&edited).map(|entry| entry.name).collect();

    assert!(!names.contains(&"user"), "{names:?}");
    assert!(!names.contains(&"guest"), "{names:?}");
    assert_eq!(names.len(), PASSWD.lines().count() - 2);
}

/// Edit: the shell of `user` cut off, which login reads as `/bin/sh`.
#[test]
fn missing_trailing_fields_read_as_empty() {
    let edited = PASSWD.replace(
        "user:x:1000:1000::/home/user.guest:/bin/bash",
        "user:x:1000:1000::/home/user.guest",
    );

    let user = passwd_entries(&edited).find(|entry| entry.name == "user");

    assert_eq!(user.map(|entry| entry.shell), Some(""));
}
