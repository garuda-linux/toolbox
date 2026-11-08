use super::{
    HomeChange, HomeDirectory, InvalidHomeDirectory, InvalidUserId, LAST_USER_ID, MAX_HOME_BYTES,
    NewHome, UserId,
};
use crate::account::tests::PASSWD;

/// The `nobody` (65534) of the passwd is above [`LAST_USER_ID`].
#[test]
fn real_home_folders_pass_and_user_ids_stop_at_the_last_one() {
    for line in PASSWD.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        let home = fields[5];

        if home != "/" {
            assert_eq!(HomeDirectory::new(home).unwrap().as_str(), home);
        }

        let id: u32 = fields[2].parse().unwrap();

        assert_eq!(
            UserId::new(id).is_ok(),
            id != 0 && id <= LAST_USER_ID,
            "{line}"
        );
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
        "/data",
    ] {
        assert_eq!(HomeDirectory::new(path).unwrap().as_str(), path);
    }
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
fn every_refusal_of_a_home_folder_has_its_own_code() {
    let codes = [
        InvalidHomeDirectory::Empty,
        InvalidHomeDirectory::TooLong { max: 1 },
        InvalidHomeDirectory::NotAbsolute,
        InvalidHomeDirectory::InvalidCharacter(':'),
        InvalidHomeDirectory::InvalidComponent,
        InvalidHomeDirectory::OuterBlank,
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
