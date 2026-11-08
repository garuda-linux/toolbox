use std::sync::Arc;

use parking_lot::Mutex;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt as _;

use super::LineLayer;

type Line = (log::Level, String, String);

fn capture(run: impl FnOnce()) -> Vec<Line> {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&lines);
    let layer = LineLayer::new(move |level, target: &str, message: &str| {
        sink.lock()
            .push((level, target.to_owned(), message.to_owned()));
    });

    tracing::subscriber::with_default(Registry::default().with(layer), run);

    lines.lock().clone()
}

fn messages(lines: &[Line]) -> Vec<&str> {
    lines.iter().map(|line| line.2.as_str()).collect()
}

#[test]
fn fields_follow_the_message() {
    let lines = capture(|| {
        tracing::info!(
            count = 3_u32,
            path = "/etc/a b",
            ok = true,
            "read the files"
        );
    });

    assert_eq!(
        lines,
        vec![(
            log::Level::Info,
            "toolbox_log::subscriber::layer::tests".to_owned(),
            r#"read the files count=3 path="/etc/a b" ok=true"#.to_owned()
        )]
    );
}

#[test]
fn the_span_path_comes_before_the_message() {
    let lines = capture(|| {
        let _apply = tracing::info_span!("apply", steps = 3_u32).entered();
        let _step = tracing::debug_span!("step", index = 2_u32, program = "pacman").entered();

        tracing::warn!("started");
    });

    assert_eq!(
        messages(&lines).first(),
        Some(&"apply{steps=3}:step{index=2 program=pacman}: started")
    );
}

#[test]
fn a_span_without_fields_is_named_alone() {
    let lines = capture(|| {
        let _read = tracing::info_span!("read").entered();

        tracing::info!(what = "dkms", "done");
    });

    assert_eq!(messages(&lines).first(), Some(&"read: done what=dkms"));
}

#[test]
fn closing_a_span_logs_how_long_it_was_open() {
    let lines = capture(|| {
        let apply = tracing::info_span!("apply", steps = 1_u32).entered();
        drop(tracing::debug_span!(parent: &apply, "step", index = 0_u32).entered());
    });

    let (level, target, message) = lines.first().unwrap();

    assert_eq!(*level, log::Level::Debug);
    assert_eq!(target, "toolbox_log::subscriber::layer::tests");
    assert!(
        message.starts_with("apply{steps=1}:step{index=0}: close elapsed="),
        "{message}"
    );
    assert!(
        messages(&lines)
            .get(1)
            .is_some_and(|line| line.starts_with("apply{steps=1}: close elapsed=")),
        "{lines:?}"
    );
}

#[test]
fn trace_spans_close_silently() {
    let lines = capture(|| {
        drop(tracing::trace_span!("frame").entered());
    });

    assert_eq!(lines, Vec::new());
}

#[test]
fn fields_recorded_later_join_the_span() {
    let lines = capture(|| {
        let span = tracing::info_span!("read", what = "units", rows = tracing::field::Empty);
        span.record("rows", 12_u32);

        span.in_scope(|| tracing::info!("listed"));
    });

    assert_eq!(
        messages(&lines).first(),
        Some(&"read{what=units rows=12}: listed")
    );
}

#[test]
fn log_records_keep_their_own_target_and_level() {
    let lines = capture(|| {
        tracing_log::format_trace(
            &log::Record::builder()
                .target("wgpu_core::device")
                .level(log::Level::Warn)
                .args(format_args!("device lost"))
                .build(),
        )
        .unwrap();
    });

    assert_eq!(
        lines,
        vec![(
            log::Level::Warn,
            "wgpu_core::device".to_owned(),
            "device lost".to_owned()
        )]
    );
}

#[test]
fn an_empty_value_is_quoted() {
    let lines = capture(|| tracing::info!(name = "", "named"));

    assert_eq!(messages(&lines), vec![r#"named name="""#]);
}
