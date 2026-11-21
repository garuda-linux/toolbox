use super::super::{SECONDARY_CONTRAST, contrast};
use super::every_tokens;

#[test]
fn every_code_ink_reads_on_the_code_fill() {
    for (theme, tokens) in every_tokens() {
        let inks = tokens.code;
        let named = [
            ("plain", inks.plain),
            ("keyword", inks.keyword),
            ("function", inks.function),
            ("string", inks.string),
            ("dim", inks.dim),
        ];

        for (name, ink) in named {
            let ratio = contrast(ink, tokens.field);
            assert!(ratio >= SECONDARY_CONTRAST, "{theme:?} {name}: {ratio}");
        }
    }
}

#[test]
fn code_kinds_read_apart() {
    for (theme, tokens) in every_tokens() {
        let inks = tokens.code;
        let coloured = [inks.keyword, inks.function, inks.string];

        for (at, ink) in coloured.iter().enumerate() {
            assert_ne!(*ink, inks.plain, "{theme:?} {at}");
            assert!(
                coloured.iter().skip(at + 1).all(|other| other != ink),
                "{theme:?} {at}"
            );
        }
    }
}
