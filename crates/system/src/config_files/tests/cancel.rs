//! A read waits for the password dialog until the caller's flag ends it.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use super::super::{ConfigFileSource as _, ReadProblem};
use super::System;
use crate::test_support::running_as_root;

#[test]
fn a_set_flag_ends_the_wait_for_the_helper_as_cancelled() {
    if running_as_root() {
        return;
    }

    let system = System::new();
    let locked = system.locked();

    // An open password dialog that nobody answers.
    system.helper("sleep 30\n");

    let started = Instant::now();

    assert_eq!(
        system
            .source()
            .read(&locked, &Arc::new(AtomicBool::new(true))),
        Err(ReadProblem::Cancelled)
    );
    assert!(started.elapsed() < Duration::from_secs(20));
}
