use crate::{LoaderDefault, LoaderEntryId, Reason};

/// As `bootctl list` showed them on the Garuda Lima image, and the name
/// snapper-sdboot gives its entries.
#[test]
fn real_entry_ids_pass() {
    for id in [
        "garuda-7.3.1-1-garuda.efi",
        "garuda-7.3.1-1-garuda-fallback.efi",
        "auto-reboot-to-firmware-setup",
        "snapper-sdboot-7.conf",
    ] {
        assert_eq!(LoaderEntryId::new(id).unwrap().as_str(), id);
    }
}

/// `loader.conf` splits a line at the first blank, so a blank or a line
/// break in an id would add a setting of its own.
#[test]
fn an_entry_id_is_one_word_of_one_line() {
    for (id, reason) in [
        ("garuda 7.efi", Reason::InvalidCharacter(' ')),
        ("garuda.efi\ntimeout 0", Reason::InvalidCharacter('\n')),
        ("-garuda.efi", Reason::InvalidStart('-')),
        ("garuda-*.efi", Reason::InvalidCharacter('*')),
        ("", Reason::Empty),
    ] {
        assert_eq!(
            LoaderEntryId::new(id).unwrap_err().reason(),
            reason,
            "{id:?}"
        );
    }
}

/// An image's entry, the glob of `loader.conf(5)`'s example and `@saved`.
#[test]
fn a_default_takes_an_id_a_glob_or_saved() {
    for default in [
        "garuda-7.3.1-1-garuda.efi",
        "01234567890abcdef1234567890abdf0-*",
        "@saved",
    ] {
        assert_eq!(LoaderDefault::new(default).unwrap().as_str(), default);
    }
}

#[test]
fn a_default_is_one_word_too() {
    for (default, reason) in [
        ("garuda *", Reason::InvalidCharacter(' ')),
        ("garuda\ttimeout", Reason::InvalidCharacter('\t')),
        ("-garuda", Reason::InvalidStart('-')),
    ] {
        assert_eq!(
            LoaderDefault::new(default).unwrap_err().reason(),
            reason,
            "{default:?}"
        );
    }
}
