use super::{ActiveState, SubCode, sub_code};

#[test]
fn a_known_sub_state_keeps_its_code() {
    assert_eq!(
        sub_code(ActiveState::Active, "exited"),
        Some(SubCode::Exited)
    );
    assert_eq!(
        sub_code(ActiveState::Active, "listening"),
        Some(SubCode::Listening)
    );
    assert_eq!(
        sub_code(ActiveState::Active, "mounted"),
        Some(SubCode::Mounted)
    );
    assert_eq!(
        sub_code(ActiveState::Active, "plugged"),
        Some(SubCode::Plugged)
    );
    assert_eq!(
        sub_code(ActiveState::Active, "waiting"),
        Some(SubCode::Waiting)
    );
    assert_eq!(
        sub_code(ActiveState::Active, "elapsed"),
        Some(SubCode::Elapsed)
    );
    assert_eq!(
        sub_code(ActiveState::Inactive, "abandoned"),
        Some(SubCode::Abandoned)
    );
}

#[test]
fn the_start_and_stop_steps_fold_into_one_code_each() {
    for sub in [
        "condition",
        "start-pre",
        "start",
        "start-post",
        "start-chown",
        "mounting",
        "mounting-done",
        "activating",
        "activating-done",
    ] {
        assert_eq!(
            sub_code(ActiveState::Failed, sub),
            Some(SubCode::Starting),
            "{sub}"
        );
    }

    for sub in [
        "stop",
        "stop-pre",
        "stop-sigterm",
        "stop-sigkill",
        "stop-post",
        "stop-watchdog",
        "final-sigterm",
        "final-sigkill",
        "final-watchdog",
        "unmounting",
        "unmounting-sigterm",
        "deactivating",
        "deactivating-sigkill",
    ] {
        assert_eq!(
            sub_code(ActiveState::Failed, sub),
            Some(SubCode::Stopping),
            "{sub}"
        );
    }
}

#[test]
fn the_reload_and_restart_steps_fold_into_one_code_each() {
    for sub in [
        "reload",
        "reload-signal",
        "reload-notify",
        "remounting",
        "remounting-sigkill",
    ] {
        assert_eq!(
            sub_code(ActiveState::Failed, sub),
            Some(SubCode::Reloading),
            "{sub}"
        );
    }

    for sub in [
        "auto-restart",
        "auto-restart-queued",
        "dead-before-auto-restart",
        "failed-before-auto-restart",
    ] {
        assert_eq!(
            sub_code(ActiveState::Failed, sub),
            Some(SubCode::Restarting),
            "{sub}"
        );
    }

    assert_eq!(
        sub_code(ActiveState::Active, "dead-resources-pinned"),
        Some(SubCode::Dead)
    );
}

#[test]
fn a_sub_state_that_repeats_the_state_is_left_out() {
    assert_eq!(sub_code(ActiveState::Active, "running"), None);
    assert_eq!(sub_code(ActiveState::Active, "active"), None);
    assert_eq!(sub_code(ActiveState::Inactive, "dead"), None);
    assert_eq!(sub_code(ActiveState::Failed, "failed"), None);
    assert_eq!(sub_code(ActiveState::Activating, "start-pre"), None);
    assert_eq!(sub_code(ActiveState::Deactivating, "stop-sigterm"), None);
    assert_eq!(sub_code(ActiveState::Reloading, "reload"), None);
}

#[test]
fn a_sub_state_newer_than_the_app_is_other_never_its_own_word() {
    assert_eq!(
        sub_code(ActiveState::Active, "frobnicating"),
        Some(SubCode::Other)
    );
    assert_eq!(
        sub_code(ActiveState::Active, "<b>bold</b>"),
        Some(SubCode::Other)
    );
}

#[test]
fn an_empty_sub_state_says_nothing() {
    assert_eq!(sub_code(ActiveState::Active, ""), None);
}

#[test]
fn the_newer_active_states_have_codes_of_their_own() {
    assert_eq!(ActiveState::parse("maintenance").as_str(), "maintenance");
    assert_eq!(ActiveState::parse("refreshing").as_str(), "refreshing");
    assert_eq!(ActiveState::parse("frobnicating").as_str(), "other");
}
