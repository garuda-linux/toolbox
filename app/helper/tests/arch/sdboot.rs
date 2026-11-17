//! The helper's bootctl read as root in the Arch test image. The image has
//! bootctl but no ESP to list, so bootctl fails or lists nothing. Either way
//! the answer must be one line the app reads, and no default may be named
//! without an answer that holds one.

use toolbox_plan::protocol::ProgramAnswer;
use toolbox_plan::sdboot::bootctl_default;

use crate::common::TestResult;
use crate::in_test_image;

#[test]
fn the_sdboot_default_as_root_is_one_line_the_app_reads() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let output = crate::oneshot::output(&["read", "sdboot-default"])?;

    assert!(output.status.success(), "{}", output.status);

    let answer = String::from_utf8(output.stdout)?;

    assert_eq!(answer.matches('\n').count(), 1);

    match ProgramAnswer::parse(answer.trim_end())? {
        ProgramAnswer::Missing => {}
        ProgramAnswer::Ran {
            output: listed,
            errors,
            code,
            truncated: _,
        } => {
            if code != Some(0_i32) {
                assert!(!errors.is_empty(), "bootctl failed without a word");
                assert_eq!(bootctl_default(&listed), None);
            }
        }
    }

    Ok(())
}
