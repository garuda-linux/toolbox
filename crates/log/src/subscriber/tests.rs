use std::ffi::OsStr;
use std::sync::Arc;

use parking_lot::Mutex;
use tracing::level_filters::LevelFilter;

use super::{Origin, Stack, build};

/// A stack whose lines land in the returned list.
fn stack(variable: Option<&OsStr>) -> (Stack, Arc<Mutex<Vec<String>>>) {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    let stack = build(LevelFilter::INFO, variable, None, move |_, _, message| {
        sink.lock().push(message.to_owned());
    });

    (stack, lines)
}

#[test]
fn the_level_changes_while_running() {
    let (Stack { subscriber, reload }, lines) = stack(None);

    tracing::subscriber::with_default(subscriber, || {
        tracing::debug!(target: "toolbox_gui::app", "hidden");
        assert!(reload.set(LevelFilter::DEBUG));
        tracing::debug!(target: "toolbox_gui::app", "shown");
    });

    assert_eq!(*lines.lock(), ["shown"]);
    assert_eq!(reload.describe(), "log level debug");
}

#[test]
fn the_variable_outlasts_a_level_change() {
    let (Stack { subscriber, reload }, lines) = stack(Some(OsStr::new("toolbox_gui=trace")));

    assert_eq!(
        reload.origin,
        Origin::Override("toolbox_gui=trace".to_owned())
    );

    tracing::subscriber::with_default(subscriber, || {
        assert!(!reload.set(LevelFilter::ERROR));
        tracing::trace!(target: "toolbox_gui::app", "still shown");
    });

    assert_eq!(*lines.lock(), ["still shown"]);
    assert_eq!(reload.describe(), "log filter toolbox_gui=trace");
}

#[test]
fn dependencies_logging_through_log_are_let_in_up_to_the_filter() {
    let reload = stack(None).0.reload;
    assert_eq!(reload.log_level(), log::LevelFilter::Info);

    reload.set(LevelFilter::TRACE);
    assert_eq!(reload.log_level(), log::LevelFilter::Trace);
}
