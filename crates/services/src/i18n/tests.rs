use std::ffi::OsString;
use std::sync::{Arc, Barrier};
use std::thread;

use super::{FALLBACK, available, language_from_env, text_in};

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let pairs: Vec<(String, String)> = pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();

    move |key| {
        pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| OsString::from(value))
    }
}

#[test]
fn english_and_german_are_on_offer() {
    let codes: Vec<String> = available().iter().map(ToString::to_string).collect();

    assert!(codes.contains(&"en-US".to_owned()));
    assert!(codes.contains(&"de-DE".to_owned()));
}

#[test]
fn the_system_locale_picks_the_language() {
    assert_eq!(
        language_from_env(env(&[("LANG", "de_DE.UTF-8")])).to_string(),
        "de-DE"
    );
}

#[test]
fn another_region_of_a_language_on_offer_gets_that_language() {
    assert_eq!(
        language_from_env(env(&[("LANG", "de_AT.UTF-8")])).to_string(),
        "de-DE"
    );
}

#[test]
fn language_list_and_lc_variables_win_over_lang() {
    assert_eq!(
        language_from_env(env(&[
            ("LANG", "en_US.UTF-8"),
            ("LC_MESSAGES", "de_DE.UTF-8")
        ]))
        .to_string(),
        "de-DE"
    );
    assert_eq!(
        language_from_env(env(&[("LANG", "en_US.UTF-8"), ("LANGUAGE", "fr:de")])).to_string(),
        "de-DE"
    );
}

#[test]
fn unknown_or_c_locales_fall_back_to_english() {
    for lang in ["C", "POSIX", "", "fr_FR.UTF-8", "C.UTF-8"] {
        assert_eq!(
            language_from_env(env(&[("LANG", lang)])).to_string(),
            "en-US",
            "{lang}"
        );
    }
}

/// Threads that miss the cache at once still get the one copy that stays.
/// Ids no file has, so no other test of this process has asked for them.
#[test]
fn threads_that_ask_for_a_new_text_at_once_get_one_copy() {
    const THREADS: usize = 8;

    for round in 0_u8..20 {
        let id = format!("i18n-test-concurrent-{round}");
        let start = Arc::new(Barrier::new(THREADS));

        let addresses: Vec<usize> = std::iter::repeat_n((), THREADS)
            .map(|()| {
                let (id, start) = (id.clone(), Arc::clone(&start));
                thread::Builder::new()
                    .name("i18n-test".to_owned())
                    .spawn(move || {
                        start.wait();
                        text_in(FALLBACK, &id).as_ptr().addr()
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();

        let cached = text_in(FALLBACK, &id).as_ptr().addr();
        assert!(
            addresses.iter().all(|address| *address == cached),
            "round {round}"
        );
    }
}
