use super::{InvalidPageName, MAX_PAGE_NAME_BYTES, PageName};

#[test]
fn a_name_is_a_short_word_of_small_letters_and_hyphens() {
    for valid in ["diagnostics", "system-settings", "inxi"] {
        assert_eq!(PageName::new(valid).unwrap().as_ref(), valid);
    }
}

#[test]
fn an_empty_or_overlong_name_is_refused() {
    assert_eq!(PageName::new(""), Err(InvalidPageName::Empty));
    PageName::new("a".repeat(MAX_PAGE_NAME_BYTES)).unwrap();
    assert_eq!(
        PageName::new("a".repeat(MAX_PAGE_NAME_BYTES + 1)),
        Err(InvalidPageName::TooLong {
            max: MAX_PAGE_NAME_BYTES
        })
    );
}

#[test]
fn capitals_paths_spaces_and_digits_are_refused() {
    for (text, found) in [
        ("Diagnostics", 'D'),
        ("../etc", '.'),
        ("collect inxi", ' '),
        ("page2", '2'),
        ("ünicode", 'ü'),
    ] {
        assert_eq!(
            PageName::new(text),
            Err(InvalidPageName::InvalidCharacter(found)),
            "{text}"
        );
    }
}

#[test]
fn a_name_parses_and_prints_as_its_text() {
    let name: PageName = "boot-tools".parse().unwrap();

    assert_eq!(name.to_string(), "boot-tools");
    assert_eq!(format!("{name:?}"), "\"boot-tools\"");
}
