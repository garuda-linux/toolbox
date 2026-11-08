use super::{DecodeError, ProgramAnswer};

#[test]
fn an_overlong_answer_is_refused_with_its_limit() {
    let line = "x".repeat(ProgramAnswer::MAX_LINE_BYTES + 1);

    match ProgramAnswer::parse(&line) {
        Err(DecodeError::TooLong { length, limit }) => {
            assert_eq!(length, line.len());
            assert_eq!(limit, ProgramAnswer::MAX_LINE_BYTES);
        }
        parsed => panic!("an overlong line parsed: {parsed:?}"),
    }
}
