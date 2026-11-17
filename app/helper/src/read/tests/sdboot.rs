//! The one program a password-less read runs: bootctl's list.

use toolbox_plan::protocol::{ProgramAnswer, ReadKind};

use super::{program, ran};

/// The one program a password-less read runs: bootctl, with its fixed
/// argument and no input, in a clean environment.
#[test]
fn sdboot_default_runs_bootctl_list_as_its_only_program() {
    let dir = tempfile::tempdir().unwrap();
    program(
        dir.path(),
        "usr/bin/bootctl",
        "read -r line || echo 'no input'; echo \"bootctl $*|$SYSTEMD_COLORS\"",
    );

    assert_eq!(
        ran(dir.path(), ReadKind::SdBootDefault),
        ProgramAnswer::Ran {
            output: "no input\nbootctl list --json=short|\n".to_owned(),
            errors: String::new(),
            code: Some(0_i32),
            truncated: false,
        }
    );
}
