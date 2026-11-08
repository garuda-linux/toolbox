use clap::Parser as _;
use toolbox_cli::Cli;

use super::{argv, chosen, defaults, pages, text};

fn command(args: &[&str]) -> toolbox_cli::args::Command {
    Cli::try_parse_from(std::iter::once("garuda-toolbox").chain(args.iter().copied()))
        .unwrap()
        .command
        .unwrap()
}

fn words(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn lists_and_shows_page() {
    for args in [
        &["services", "list"][..],
        &["toggle", "list", "--on"],
        &["apps", "list"],
        &["kernels", "list"],
        &["locales", "packs", "list"],
        &["users", "list"],
        &["configs", "show", "pacman-config"],
        &["network", "show"],
        &["network", "drivers", "list"],
        &["boot", "show"],
        &["boot", "entries"],
        &["bootrepair", "show"],
        &["settings", "list"],
        &["pages"],
        &["network", "public-address"],
    ] {
        assert!(pages(&command(args)), "{args:?}");
    }
}

#[test]
fn json_questions_and_running_programs_never_page() {
    for args in [
        &["services", "list", "--json"][..],
        &["boot", "show", "--json"],
        &["network", "show", "--json"],
        &["services", "start", "sshd.service"],
        &["toggle", "on", "steam"],
        &["configs", "edit", "pacman-config"],
        &["network", "ping", "garudalinux.org"],
        &["network", "traceroute", "garudalinux.org"],
        &["update"],
        &["diagnostics", "inxi"],
        &["status"],
        &["status", "--watch"],
    ] {
        assert!(!pages(&command(args)), "{args:?}");
    }
}

#[test]
fn the_own_variable_comes_before_pager_and_less_is_last() {
    assert_eq!(
        argv(Some("most -s"), Some("more")),
        Some(words(&["most", "-s"]))
    );
    assert_eq!(argv(None, Some("more")), Some(words(&["more"])));
    assert_eq!(argv(None, None), Some(words(&["less", "-FRX"])));
}

#[test]
fn cat_or_an_empty_value_prints_directly() {
    assert_eq!(argv(Some(""), Some("less")), None);
    assert_eq!(argv(Some("  "), None), None);
    assert_eq!(argv(None, Some("cat")), None);
    assert_eq!(argv(Some(" cat "), None), None);
}

#[test]
fn a_value_is_split_into_words_without_a_shell() {
    assert_eq!(
        argv(Some("less -R  --quit-if-one-screen; rm"), None),
        Some(words(&["less", "-R", "--quit-if-one-screen;", "rm"]))
    );
}

#[test]
fn no_terminal_or_no_pager_prints_directly() {
    let list = command(&["services", "list"]);

    assert_eq!(
        chosen(&list, false, true, None, None),
        Some(words(&["less", "-FRX"]))
    );
    assert_eq!(chosen(&list, true, true, None, None), None);
    assert_eq!(chosen(&list, false, false, None, None), None);
    assert_eq!(chosen(&command(&["update"]), false, true, None, None), None);
}

#[test]
fn a_value_that_is_no_text_turns_the_pager_off() {
    use std::os::unix::ffi::OsStrExt as _;

    let odd = std::ffi::OsStr::from_bytes(b"less\xff");
    assert_eq!(text(odd), "");
    assert_eq!(text(std::ffi::OsStr::new("most")), "most");
}

#[test]
fn doctor_prints_directly_as_status_does() {
    assert!(!pages(&command(&["doctor"])));
    assert!(!pages(&command(&["doctor", "--json"])));
}

#[test]
fn less_and_lv_get_colours_when_unset_as_git_does() {
    assert_eq!(defaults(None, None), [("LESS", "FRX"), ("LV", "-c")]);
}

#[test]
fn a_set_less_or_lv_stays_as_it_is() {
    let own = std::ffi::OsStr::new("-S");

    assert_eq!(defaults(Some(own), None), [("LV", "-c")]);
    assert_eq!(defaults(None, Some(own)), [("LESS", "FRX")]);
    assert_eq!(defaults(Some(std::ffi::OsStr::new("")), Some(own)), []);
}
