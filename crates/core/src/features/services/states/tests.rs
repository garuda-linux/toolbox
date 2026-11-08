use super::{ActiveState, sub_code};

#[test]
fn a_known_sub_state_keeps_its_code() {
    assert_eq!(sub_code(ActiveState::Active, "exited"), "exited");
    assert_eq!(sub_code(ActiveState::Active, "listening"), "listening");
    assert_eq!(sub_code(ActiveState::Active, "mounted"), "mounted");
    assert_eq!(sub_code(ActiveState::Active, "plugged"), "plugged");
    assert_eq!(sub_code(ActiveState::Active, "waiting"), "waiting");
    assert_eq!(sub_code(ActiveState::Active, "elapsed"), "elapsed");
    assert_eq!(sub_code(ActiveState::Inactive, "abandoned"), "abandoned");
}

#[test]
fn the_steps_of_a_change_fold_into_one_code() {
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
        assert_eq!(sub_code(ActiveState::Failed, sub), "starting", "{sub}");
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
        assert_eq!(sub_code(ActiveState::Failed, sub), "stopping", "{sub}");
    }

    for sub in [
        "reload",
        "reload-signal",
        "reload-notify",
        "remounting",
        "remounting-sigkill",
    ] {
        assert_eq!(sub_code(ActiveState::Failed, sub), "reloading", "{sub}");
    }

    for sub in [
        "auto-restart",
        "auto-restart-queued",
        "dead-before-auto-restart",
        "failed-before-auto-restart",
    ] {
        assert_eq!(sub_code(ActiveState::Failed, sub), "restarting", "{sub}");
    }

    assert_eq!(
        sub_code(ActiveState::Active, "dead-resources-pinned"),
        "dead"
    );
}

#[test]
fn a_sub_state_that_repeats_the_state_is_left_out() {
    assert_eq!(sub_code(ActiveState::Active, "running"), "");
    assert_eq!(sub_code(ActiveState::Active, "active"), "");
    assert_eq!(sub_code(ActiveState::Inactive, "dead"), "");
    assert_eq!(sub_code(ActiveState::Failed, "failed"), "");
    assert_eq!(sub_code(ActiveState::Activating, "start-pre"), "");
    assert_eq!(sub_code(ActiveState::Deactivating, "stop-sigterm"), "");
    assert_eq!(sub_code(ActiveState::Reloading, "reload"), "");
}

#[test]
fn a_sub_state_newer_than_the_app_is_other_never_its_own_word() {
    assert_eq!(sub_code(ActiveState::Active, "frobnicating"), "other");
    assert_eq!(sub_code(ActiveState::Active, "<b>bold</b>"), "other");
}

#[test]
fn an_empty_sub_state_says_nothing() {
    assert_eq!(sub_code(ActiveState::Active, ""), "");
}

#[test]
fn the_newer_active_states_have_codes_of_their_own() {
    assert_eq!(ActiveState::parse("maintenance").as_str(), "maintenance");
    assert_eq!(ActiveState::parse("refreshing").as_str(), "refreshing");
    assert_eq!(ActiveState::parse("frobnicating").as_str(), "other");
}
