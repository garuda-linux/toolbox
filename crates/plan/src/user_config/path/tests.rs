use super::{ConfigWord, InvalidConfigWord, MAX_WORD_BYTES};

#[test]
fn an_overlong_word_is_refused_with_its_limit() {
    assert_eq!(
        ConfigWord::new("x".repeat(MAX_WORD_BYTES + 1)).unwrap_err(),
        InvalidConfigWord::TooLong {
            max: MAX_WORD_BYTES,
        }
    );
}
