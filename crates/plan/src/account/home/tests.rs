use super::{
    HomeChange, HomeDirectory, InvalidHomeDirectory, InvalidUserId, LAST_USER_ID, MAX_HOME_BYTES,
    NewHome, UserId,
};
use crate::account::tests::PASSWD;

/// Every line of the captured passwd: its user id, its home and the
/// outcome of each, in the order of the file.
const CAPTURED: [(u32, &str, Result<&str, InvalidHomeDirectory>, bool); 27] = [
    (0, "/root", Err(InvalidHomeDirectory::OutsideRoots), false),
    (1, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (2, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (
        8,
        "/var/spool/mail",
        Err(InvalidHomeDirectory::OutsideRoots),
        true,
    ),
    (14, "/srv/ftp", Ok("/srv/ftp"), true),
    (33, "/srv/http", Ok("/srv/http"), true),
    (
        0xFFFE,
        "/",
        Err(InvalidHomeDirectory::InvalidComponent),
        false,
    ),
    (81, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (979, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (978, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (977, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (976, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (975, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (974, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (973, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (972, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (971, "/var/lib/libuuid", Ok("/var/lib/libuuid"), true),
    (970, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (969, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (40, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (968, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (32, "/var/lib/rpcbind", Ok("/var/lib/rpcbind"), true),
    (34, "/var/lib/nfs", Ok("/var/lib/nfs"), true),
    (1000, "/home/user.guest", Ok("/home/user.guest"), true),
    (967, "/", Err(InvalidHomeDirectory::InvalidComponent), true),
    (1001, "/home/guest", Ok("/home/guest"), true),
    (1002, "/home/nopass", Ok("/home/nopass"), true),
];

#[test]
fn captured_passwd_homes_and_user_ids_have_their_outcome() {
    assert_eq!(PASSWD.lines().count(), CAPTURED.len());

    for (line, (uid, home, outcome, uid_ok)) in PASSWD.lines().zip(CAPTURED) {
        let fields: Vec<&str> = line.split(':').collect();

        assert_eq!(fields[2], uid.to_string(), "{line}");
        assert_eq!(fields[5], home, "{line}");
        assert_eq!(
            HomeDirectory::new(home).map(|home| home.as_str().to_owned()),
            outcome.map(str::to_owned),
            "{line}"
        );
        assert_eq!(UserId::new(uid).is_ok(), uid_ok, "{line}");
    }
}

/// Paths a user types for a new home, for what no captured passwd has.
#[test]
fn home_folders_are_plain_absolute_paths() {
    for path in [
        "/home/ada",
        "/srv/people/ada",
        "/home/Ada Lovelace",
        "/home/josé",
        "/var/lib/-odd",
        "/opt/ada",
    ] {
        assert_eq!(HomeDirectory::new(path).unwrap().as_str(), path);
    }
}

/// A home in a tree root trusts would hand the user files there, one
/// below a shared folder races the check against another user.
#[test]
fn home_folders_outside_the_roots_are_refused() {
    for path in [
        "/etc/systemd/system/x.service.d",
        "/tmp/ada",
        "/var/tmp/ada",
        "/dev/shm/ada",
        "/root",
        "/data",
        "/home",
        "/srv",
        "/var/lib",
        "/opt",
        "/homes/ada",
        "/optional/ada",
        "/var/library/ada",
        "/usr/lib/ada",
    ] {
        assert_eq!(
            HomeDirectory::new(path),
            Err(InvalidHomeDirectory::OutsideRoots),
            "{path}"
        );
    }

    serde_json::from_str::<HomeDirectory>("\"/etc/cron.d/ada\"").unwrap_err();
}

#[test]
fn home_folders_refuse_what_climbs_or_breaks_passwd() {
    for (path, error) in [
        ("", InvalidHomeDirectory::Empty),
        ("home/ada", InvalidHomeDirectory::NotAbsolute),
        ("~/ada", InvalidHomeDirectory::NotAbsolute),
        ("-m", InvalidHomeDirectory::NotAbsolute),
        ("/", InvalidHomeDirectory::InvalidComponent),
        ("/home/../etc", InvalidHomeDirectory::InvalidComponent),
        ("/home/ada/..", InvalidHomeDirectory::InvalidComponent),
        ("/home/./ada", InvalidHomeDirectory::InvalidComponent),
        ("/home//ada", InvalidHomeDirectory::InvalidComponent),
        ("/home/ada/", InvalidHomeDirectory::InvalidComponent),
        ("/home/a:b", InvalidHomeDirectory::InvalidCharacter(':')),
        (
            "/home/ada\nroot::0:0",
            InvalidHomeDirectory::InvalidCharacter('\n'),
        ),
        ("/home/a\tb", InvalidHomeDirectory::InvalidCharacter('\t')),
        ("/home/a\0b", InvalidHomeDirectory::InvalidCharacter('\0')),
        (
            "/home/a\u{1b}[0m",
            InvalidHomeDirectory::InvalidCharacter('\u{1b}'),
        ),
        ("/home/ada ", InvalidHomeDirectory::OuterBlank),
        (" /home/ada", InvalidHomeDirectory::OuterBlank),
    ] {
        assert_eq!(
            HomeDirectory::new(path),
            Err(error),
            "{}",
            path.escape_debug()
        );
    }

    assert_eq!(
        HomeDirectory::new(format!("/{}", "x".repeat(MAX_HOME_BYTES))),
        Err(InvalidHomeDirectory::TooLong {
            max: MAX_HOME_BYTES
        })
    );
}

#[test]
fn a_home_folder_hides_no_character() {
    let line = PASSWD
        .lines()
        .find(|line| line.starts_with("user:"))
        .unwrap();

    let home = line.split(':').nth(5).unwrap();

    for hidden in ['\u{202e}', '\u{200d}'] {
        // The captured home with the character after its last /.
        let spoofed = home.replacen("/user", &format!("/{hidden}user"), 1);

        assert_eq!(
            HomeDirectory::new(spoofed),
            Err(InvalidHomeDirectory::InvalidCharacter(hidden))
        );
    }
}

#[test]
fn every_refusal_of_a_home_folder_has_its_own_code() {
    let codes = [
        InvalidHomeDirectory::Empty,
        InvalidHomeDirectory::TooLong { max: 1 },
        InvalidHomeDirectory::NotAbsolute,
        InvalidHomeDirectory::InvalidCharacter(':'),
        InvalidHomeDirectory::InvalidComponent,
        InvalidHomeDirectory::OuterBlank,
        InvalidHomeDirectory::OutsideRoots,
    ]
    .map(InvalidHomeDirectory::as_str);

    let unique: std::collections::BTreeSet<&str> = codes.into_iter().collect();

    assert_eq!(unique.len(), codes.len());
    assert!(codes.iter().all(|code| code.starts_with("home-")));
}

#[test]
fn user_ids_are_never_root_and_never_above_the_last_one() {
    assert_eq!(UserId::new(0), Err(InvalidUserId(0)));
    assert_eq!(
        UserId::new(LAST_USER_ID + 1),
        Err(InvalidUserId(LAST_USER_ID + 1))
    );
    assert_eq!(UserId::new(u32::MAX), Err(InvalidUserId(u32::MAX)));
    assert_eq!(UserId::new(1).unwrap().get(), 1);
    assert_eq!(UserId::new(LAST_USER_ID).unwrap().get(), LAST_USER_ID);
    assert!(UserId::new(999).unwrap().is_system());
    assert!(!UserId::new(1000).unwrap().is_system());
    assert_eq!(UserId::new(1234).unwrap().to_string(), "1234");
}

#[test]
fn decoding_checks_paths_and_numbers_again() {
    assert_eq!(
        serde_json::from_str::<HomeDirectory>("\"/home/ada\"").unwrap(),
        HomeDirectory::new("/home/ada").unwrap()
    );
    serde_json::from_str::<HomeDirectory>("\"/home/../root\"").unwrap_err();
    serde_json::from_str::<HomeDirectory>("\"home\"").unwrap_err();
    serde_json::from_str::<UserId>("0").unwrap_err();
    serde_json::from_str::<UserId>("-5").unwrap_err();
    serde_json::from_str::<UserId>("70000").unwrap_err();
    serde_json::from_str::<UserId>("\"1000\"").unwrap_err();
    assert_eq!(serde_json::from_str::<UserId>("1500").unwrap().get(), 1500);
}

#[test]
fn a_new_home_is_one_of_three_and_a_change_names_its_folder() {
    let at = NewHome::At(HomeDirectory::new("/srv/ada").unwrap());

    assert_eq!(
        serde_json::to_string(&NewHome::Standard).unwrap(),
        "{\"kind\":\"standard\"}"
    );
    assert_eq!(
        serde_json::to_string(&NewHome::Absent).unwrap(),
        "{\"kind\":\"absent\"}"
    );
    assert_eq!(
        serde_json::to_string(&at).unwrap(),
        "{\"kind\":\"at\",\"path\":\"/srv/ada\"}"
    );
    assert_eq!(
        serde_json::from_str::<NewHome>("{\"kind\":\"at\",\"path\":\"/srv/ada\"}").unwrap(),
        at
    );
    serde_json::from_str::<NewHome>("{\"kind\":\"at\",\"path\":\"srv\"}").unwrap_err();
    serde_json::from_str::<NewHome>("{\"kind\":\"at\"}").unwrap_err();
    serde_json::from_str::<NewHome>("{\"kind\":\"shared\"}").unwrap_err();
    serde_json::from_str::<HomeChange>("{\"home\":\"/a/..\",\"move_files\":true}").unwrap_err();
    serde_json::from_str::<HomeChange>("{\"home\":\"/srv/ada\"}").unwrap_err();
    assert_eq!(NewHome::default(), NewHome::Standard);
}

#[test]
fn a_home_folder_parses_from_str_and_borrows_as_str() {
    let parsed: HomeDirectory = "/home/ada".parse().unwrap();

    assert_eq!(parsed.as_ref(), "/home/ada");
    assert_eq!(parsed.to_string(), "/home/ada");
    "/home/ada\0".parse::<HomeDirectory>().unwrap_err();
}

#[test]
fn user_ids_serialize_as_numbers() {
    assert_eq!(
        serde_json::to_string(&UserId::new(1000).unwrap()).unwrap(),
        "1000"
    );
}
