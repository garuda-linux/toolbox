use super::{InvalidRgb, InvalidSchemeId, MAX_SCHEME_ID_BYTES, Rgb, SchemeId};

#[test]
fn a_scheme_id_is_lowercase_words_joined_by_dashes() {
    for valid in ["nord", "catppuccin-mocha", "tokyo-night-day", "base16"] {
        assert_eq!(SchemeId::new(valid).unwrap().as_str(), valid);
    }
}

#[test]
fn odd_scheme_ids_are_refused() {
    assert_eq!(SchemeId::new(""), Err(InvalidSchemeId::Empty));
    for odd in [
        "Nord",
        "nord-",
        "-nord",
        "nord--light",
        "nord light",
        "../nord",
    ] {
        assert_eq!(SchemeId::new(odd), Err(InvalidSchemeId::NotWords), "{odd}");
    }

    SchemeId::new("a".repeat(MAX_SCHEME_ID_BYTES)).unwrap();
    assert_eq!(
        SchemeId::new("a".repeat(MAX_SCHEME_ID_BYTES + 1)),
        Err(InvalidSchemeId::TooLong {
            max: MAX_SCHEME_ID_BYTES
        })
    );
}

#[test]
fn a_scheme_id_reads_and_writes_as_its_text() {
    let id: SchemeId = serde_json::from_str("\"nord\"").unwrap();
    assert_eq!(serde_json::to_string(&id).unwrap(), "\"nord\"");
    serde_json::from_str::<SchemeId>("\"Nord\"").unwrap_err();
}

#[test]
fn a_colour_is_a_hash_and_six_hex_digits() {
    assert_eq!(Rgb::parse("#1e1e2e").unwrap().bytes(), [0x1e, 0x1e, 0x2e]);
    assert_eq!(Rgb::parse("#CBA6F7").unwrap().bytes(), [0xcb, 0xa6, 0xf7]);
    assert_eq!(Rgb::parse("#cba6f7").unwrap().to_string(), "#cba6f7");
}

#[test]
fn other_colour_texts_are_refused() {
    for odd in ["", "1e1e2e", "#1e1e2", "#1e1e2e0", "#1e1g2e", "#ééé", "red"] {
        assert_eq!(Rgb::parse(odd), Err(InvalidRgb(odd.to_owned())), "{odd}");
    }
}
