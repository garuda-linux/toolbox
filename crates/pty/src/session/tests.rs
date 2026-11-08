use std::os::unix::process::ExitStatusExt as _;
use std::process::ExitStatus;

use super::Exit;

#[test]
fn exits_keep_their_code_or_signal() {
    for (raw, exit) in [
        (0_i32, Exit::Code(0)),
        (1_i32 << 8_i32, Exit::Code(1)),
        (2_i32, Exit::Signal(2)),
        (9_i32, Exit::Signal(9)),
    ] {
        assert_eq!(Exit::from(ExitStatus::from_raw(raw)), exit, "{raw}");
    }
}
