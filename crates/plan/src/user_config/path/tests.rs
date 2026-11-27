use super::{ConfigWord, HomePath, InvalidConfigWord, InvalidHomePath, MAX_WORD_BYTES};

#[test]
fn an_overlong_word_is_refused_with_its_limit() {
    assert_eq!(
        ConfigWord::new("x".repeat(MAX_WORD_BYTES + 1)).unwrap_err(),
        InvalidConfigWord::TooLong {
            max: MAX_WORD_BYTES,
        }
    );
}

#[test]
fn a_home_path_refuses_a_nul_byte_by_name() {
    assert_eq!(
        HomePath::new("a\0b"),
        Err(InvalidHomePath::InvalidCharacter('\0'))
    );
}
