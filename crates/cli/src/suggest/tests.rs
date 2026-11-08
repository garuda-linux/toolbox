use super::{did_you_mean, suggest};

const UNITS: [&str; 3] = ["bluetooth.service", "sshd.service", "cups.socket"];

#[test]
fn a_typo_finds_the_close_name() {
    assert_eq!(
        suggest("blutooth.service", UNITS),
        Some("bluetooth.service")
    );
    assert_eq!(
        suggest("linux-zem", ["linux", "linux-lts", "linux-zen"]),
        Some("linux-zen")
    );
}

#[test]
fn case_does_not_count() {
    assert_eq!(suggest("Firefox", ["chromium", "firefox"]), Some("firefox"));
}

#[test]
fn a_missing_service_suffix_is_found() {
    assert_eq!(suggest("sshd", UNITS), Some("sshd.service"));
    assert_eq!(suggest("blutooth", UNITS), Some("bluetooth.service"));
}

#[test]
fn a_far_name_is_no_suggestion() {
    assert_eq!(suggest("docker", UNITS), None);
    assert_eq!(suggest("x", ["y", "z"]), None);
    assert_eq!(suggest("anything", []), None);
}

#[test]
fn the_closest_of_several_wins() {
    assert_eq!(
        suggest("de_DE.UTF8", ["de_AT.UTF-8", "de_DE.UTF-8"]),
        Some("de_DE.UTF-8")
    );
}

#[test]
fn the_hint_names_the_suggestion() {
    assert_eq!(
        did_you_mean("sshd", UNITS).as_deref(),
        Some("did you mean `sshd.service`?")
    );
    assert_eq!(did_you_mean("docker", UNITS), None);
}

#[test]
fn two_swapped_letters_are_one_typo() {
    assert_eq!(suggest("audoi", ["audio", "video"]), Some("audio"));
    assert_eq!(suggest("sshd.serivce", UNITS), Some("sshd.service"));
}
