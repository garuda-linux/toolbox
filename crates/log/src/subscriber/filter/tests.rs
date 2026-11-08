use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::sync::Arc;

use parking_lot::Mutex;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::{EnvFilter, Layer as _, Registry};

use super::{Origin, chosen, describe, profiler_filter, settings_filter};
use crate::subscriber::layer::LineLayer;

macro_rules! every_level {
    ($target:literal) => {
        tracing::error!(target: $target, "error");
        tracing::warn!(target: $target, "warn");
        tracing::info!(target: $target, "info");
        tracing::debug!(target: $target, "debug");
        tracing::trace!(target: $target, "trace");
    };
}

fn passing(filter: EnvFilter) -> Vec<String> {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    let layer = LineLayer::new(move |level: log::Level, target: &str, _: &str| {
        sink.lock()
            .push(format!("{target} {}", level.as_str().to_ascii_lowercase()));
    });

    let subscriber = Registry::default().with(layer.with_filter(filter));

    tracing::subscriber::with_default(subscriber, || {
        every_level!("toolbox_gui::app");
        every_level!("garuda_toolbox");
        every_level!("zbus::connection");
        every_level!("wgpu_hal::vulkan::instance");
    });

    lines.lock().clone()
}

fn levels<'a>(passed: &'a [String], target: &str) -> Vec<&'a str> {
    passed
        .iter()
        .filter_map(|line| line.strip_prefix(target)?.strip_prefix(' '))
        .collect()
}

#[test]
fn info_keeps_dependencies_at_warn() {
    let passed = passing(settings_filter(LevelFilter::INFO));

    assert_eq!(
        levels(&passed, "toolbox_gui::app"),
        ["error", "warn", "info"]
    );
    assert_eq!(levels(&passed, "garuda_toolbox"), ["error", "warn", "info"]);
    assert_eq!(levels(&passed, "zbus::connection"), ["error", "warn"]);
}

#[test]
fn error_and_warn_keep_dependencies_at_warn() {
    let passed = passing(settings_filter(LevelFilter::ERROR));
    assert_eq!(levels(&passed, "toolbox_gui::app"), ["error"]);
    assert_eq!(levels(&passed, "zbus::connection"), ["error", "warn"]);

    let warned = passing(settings_filter(LevelFilter::WARN));
    assert_eq!(levels(&warned, "toolbox_gui::app"), ["error", "warn"]);
    assert_eq!(levels(&warned, "zbus::connection"), ["error", "warn"]);
}

#[test]
fn debug_is_ours_alone() {
    let passed = passing(settings_filter(LevelFilter::DEBUG));

    assert_eq!(
        levels(&passed, "toolbox_gui::app"),
        ["error", "warn", "info", "debug"]
    );
    assert_eq!(levels(&passed, "zbus::connection"), ["error", "warn"]);
}

#[test]
fn trace_opens_dependencies_to_info() {
    let passed = passing(settings_filter(LevelFilter::TRACE));

    assert_eq!(
        levels(&passed, "garuda_toolbox"),
        ["error", "warn", "info", "debug", "trace"]
    );
    assert_eq!(
        levels(&passed, "zbus::connection"),
        ["error", "warn", "info"]
    );
}

#[test]
fn the_variable_replaces_the_settings() {
    let (filter, origin) = chosen(
        LevelFilter::INFO,
        Some(OsStr::new("zbus=trace,toolbox_gui=debug")),
    );

    let passed = passing(filter);

    assert_eq!(
        origin,
        Origin::Override("zbus=trace,toolbox_gui=debug".to_owned())
    );
    assert_eq!(
        levels(&passed, "zbus::connection"),
        ["error", "warn", "info", "debug", "trace"]
    );
    assert_eq!(
        levels(&passed, "toolbox_gui::app"),
        ["error", "warn", "info", "debug"]
    );
}

#[test]
fn a_bad_variable_falls_back_to_the_settings() {
    let (filter, origin) = chosen(LevelFilter::DEBUG, Some(OsStr::new("zbus=loud")));

    assert!(
        matches!(&origin, Origin::Rejected { value, error: _ } if value == "zbus=loud"),
        "{origin:?}"
    );
    assert_eq!(
        passing(filter),
        passing(settings_filter(LevelFilter::DEBUG))
    );
}

#[test]
fn a_variable_that_is_no_text_is_rejected() {
    let (_, origin) = chosen(LevelFilter::INFO, Some(OsStr::from_bytes(b"zbus=\xff")));

    assert!(
        matches!(origin, Origin::Rejected { value: _, error: _ }),
        "{origin:?}"
    );
}

#[test]
fn an_empty_variable_counts_as_unset() {
    let (_, origin) = chosen(LevelFilter::INFO, Some(OsStr::new("")));
    assert_eq!(origin, Origin::Settings);

    let (_, unset) = chosen(LevelFilter::INFO, None);
    assert_eq!(unset, Origin::Settings);
}

#[test]
fn the_profiler_sees_every_level_of_ours() {
    let passed = passing(profiler_filter());

    assert_eq!(
        levels(&passed, "toolbox_gui::app"),
        ["error", "warn", "info", "debug", "trace"]
    );
    assert_eq!(levels(&passed, "zbus::connection"), ["error", "warn"]);
}

#[test]
fn the_start_line_names_where_the_filter_came_from() {
    assert_eq!(
        describe(LevelFilter::DEBUG, &Origin::Settings),
        "log level debug"
    );
    assert_eq!(
        describe(
            LevelFilter::DEBUG,
            &Origin::Override("zbus=trace".to_owned())
        ),
        "log filter zbus=trace"
    );
}

#[test]
fn the_vulkan_loader_only_logs_errors_until_everything_is_asked_for() {
    for level in [LevelFilter::INFO, LevelFilter::DEBUG] {
        let passed = passing(settings_filter(level));
        assert_eq!(levels(&passed, "wgpu_hal::vulkan::instance"), ["error"]);
    }

    let everything = passing(settings_filter(LevelFilter::TRACE));
    assert_eq!(
        levels(&everything, "wgpu_hal::vulkan::instance"),
        ["error", "warn", "info"]
    );
}
