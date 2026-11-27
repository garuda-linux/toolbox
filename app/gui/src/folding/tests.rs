//! `Needle` of the lists and `matches` of the highlight must find the same
//! hits, including the final sigma and the dotted capital I.

use toolbox_core::features::search::Needle;
use toolbox_gui_kit::widgets::matches;

/// The text, the search and whether the search is in the text.
const CASES: [(&str, &str, bool); 10] = [
    ("Wireless-AC 9560", "AC 95", true),
    ("Ethernet", "xyz", false),
    // The final sigma folds to the plain one on both sides.
    ("ΟΔΟΣ", "οδος", true),
    ("οδος", "ΟΔΟΣ", true),
    ("ΟΔΟΣ ΚΑΙ", "ΚΑΙΣ", false),
    // Lower case of the dotted capital I is two letters.
    ("İstanbul", "i", true),
    ("İstanbul", "İ", true),
    ("Istanbul", "İ", false),
    ("Straße", "SS", false),
    ("Straße", "ß", true),
];

#[test]
fn the_search_and_the_highlight_find_the_same_hits() {
    for (text, query, found) in CASES {
        let needle = Needle::new(query);
        assert_eq!(needle.is_in(text), found, "Needle, {text:?} for {query:?}");
        assert_eq!(
            !matches(text, query).is_empty(),
            found,
            "highlight, {text:?} for {query:?}"
        );
    }
}
