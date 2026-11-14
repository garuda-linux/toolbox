use super::RefusalText as _;
use crate::features::{kernels, users};

#[test]
fn the_code_is_the_kebab_name() {
    assert_eq!(kernels::Refusal::LastKernel.code(), "last-kernel");
}

#[test]
fn the_text_id_joins_the_prefix_and_the_code() {
    assert_eq!(
        kernels::Refusal::RunningUnknown.text_id(),
        "kernels-refusal-running-unknown"
    );
    assert_eq!(
        users::Refusal::HomeTaken.text_id(),
        "users-refused-home-taken"
    );
}
