//! The helper's program reads as root in the Arch test image: `dmesg` and
//! `journalctl` are installed there, but a container may keep the kernel
//! log even from root and has no journal. Both must come back as a clean
//! answer.

use toolbox_plan::protocol::{MAX_READ_OUTPUT_BYTES, ProgramAnswer};

use crate::common::TestResult;
use crate::in_test_image;

fn read(kind: &str) -> Result<ProgramAnswer, Box<dyn std::error::Error>> {
    let output = crate::oneshot::output(&["logs", kind])?;

    let answer = String::from_utf8(output.stdout)?;

    assert!(output.status.success(), "{kind}: {}", output.status);
    assert_eq!(answer.matches('\n').count(), 1, "{kind}");
    Ok(ProgramAnswer::parse(answer.trim_end())?)
}

#[test]
fn dmesg_as_root_gives_the_kernel_log_or_says_why_not() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    match read("dmesg")? {
        ProgramAnswer::Missing => panic!("util-linux is part of the image"),
        ProgramAnswer::Ran {
            output,
            errors,
            code,
            truncated: _,
        } => {
            assert!(output.len() <= MAX_READ_OUTPUT_BYTES);

            if code == Some(0_i32) {
                // The helper asks for colours, which the app shows.
                assert!(
                    output.is_empty() || output.contains('\u{1b}'),
                    "dmesg printed no colours"
                );
            } else {
                // A container without CAP_SYSLOG.
                assert!(errors.contains("dmesg:"), "{errors}");
            }
        }
    }

    Ok(())
}

#[test]
fn the_journal_as_root_answers_without_a_running_systemd() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    match read("journal")? {
        ProgramAnswer::Missing => panic!("systemd is part of the image"),
        ProgramAnswer::Ran {
            output,
            errors,
            code,
            truncated,
        } => {
            assert!(code.is_some(), "journalctl was killed");
            assert!(!truncated, "the last 1000 lines are below the limit");
            assert!(output.lines().count() <= 1001, "more than `-e` gives");

            // Without journal files journalctl says so on either stream.
            assert!(
                !output.is_empty() || !errors.is_empty(),
                "journalctl said nothing"
            );
        }
    }

    Ok(())
}
