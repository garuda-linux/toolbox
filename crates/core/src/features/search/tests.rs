use super::{Needle, Searchable, filter};

const ROWS: [Row; 3] = [
    Row {
        name: "wheel",
        id: "998",
    },
    Row {
        name: "audio",
        id: "996",
    },
    Row {
        name: "Docker",
        id: "960",
    },
];

/// A row found by its name or its id.
struct Row {
    name: &'static str,
    id: &'static str,
}

impl Searchable for Row {
    fn texts(&self) -> impl Iterator<Item = &str> {
        [self.name, self.id].into_iter()
    }
}

fn lowered(text: &str, search: &str) -> bool {
    text.to_lowercase().contains(&search.trim().to_lowercase())
}

#[test]
fn a_search_is_found_whatever_the_case_of_either_side() {
    for (text, search) in [
        ("NetworkManager.service", "networkm"),
        ("networkmanager.service", "NetworkM"),
        ("Deutsch (Österreich)", "österr"),
        ("deutsch (österreich)", "ÖSTERR"),
        ("Ελληνικά", "ΛΛΗΝ"),
        ("sshd.service", "  SSHD "),
        ("anything", ""),
        ("", ""),
    ] {
        assert!(lowered(text, search), "{search:?} in the lowered {text:?}");
        assert!(Needle::new(search).is_in(text), "{search:?} in {text:?}");
    }
}

#[test]
fn what_is_not_there_is_not_found() {
    for (text, search) in [
        ("sshd.service", "sshdd"),
        ("sshd", "sshd.service"),
        ("", "a"),
        ("abcab", "abd"),
    ] {
        assert!(
            !lowered(text, search),
            "{search:?} not in the lowered {text:?}"
        );
        assert!(!Needle::new(search).is_in(text), "{search:?} in {text:?}");
    }
}

/// `İ` is `i` and a combining dot in lower case. A search for a part of
/// such a letter finds it, as in the lowered text.
#[test]
fn a_letter_that_is_several_in_lower_case_is_compared_as_those() {
    assert_eq!("İ".to_lowercase(), "i\u{307}");

    for (text, search) in [
        ("İstanbul", "i\u{307}stan"),
        ("İstanbul", "istanbul"),
        ("İstanbul", "\u{307}s"),
        ("Diyarbakır İli", "İ"),
        ("aİ", "ai"),
    ] {
        assert_eq!(
            Needle::new(search).is_in(text),
            lowered(text, search),
            "{search:?} in {text:?}"
        );
    }

    assert!(!Needle::new("istanbul").is_in("İstanbul"));
    assert!(Needle::new("\u{307}s").is_in("İstanbul"));
}

/// The one difference to lowering both sides: a sigma at the end of a
/// word is the same letter as one inside. Lowering makes `Σ` a `ς` there
/// and a `σ` elsewhere, so a search for a part of a word misses it.
#[test]
fn both_lower_case_sigmas_are_one_letter() {
    assert_eq!("ΟΔΟΣ".to_lowercase(), "οδος");
    assert!(lowered("ΟΔΟΣ", "οδος"));
    assert!(Needle::new("οδος").is_in("ΟΔΟΣ"));
    assert!(Needle::new("ΟΔΟΣ").is_in("οδος"));

    assert!(!lowered("ΟΔΟΣ ΑΘΗΝΑΣ", "οδοσ"));
    assert!(Needle::new("οδοσ").is_in("ΟΔΟΣ ΑΘΗΝΑΣ"));
    assert!(!lowered("οδοσ", "ΟΔΟΣ"));
    assert!(Needle::new("ΟΔΟΣ").is_in("οδοσ"));
}

#[test]
fn only_blanks_are_no_search() {
    assert!(Needle::new("  \t").is_empty());
    assert!(!Needle::new(" a ").is_empty());
}

fn found(search: &str) -> Vec<&'static str> {
    filter(&ROWS, &Needle::new(search))
        .map(|row| row.name)
        .collect()
}

#[test]
fn a_row_is_found_by_any_of_its_texts() {
    assert_eq!(found("DOCK"), ["Docker"]);
    assert_eq!(found("99"), ["wheel", "audio"]);
    assert_eq!(found("6"), ["audio", "Docker"]);
    assert_eq!(found("users"), Vec::<&str>::new());
}

#[test]
fn an_empty_search_finds_every_row_in_its_order() {
    assert_eq!(found("  "), ["wheel", "audio", "Docker"]);
    assert!(Needle::default().finds(&ROWS[0]));
}
