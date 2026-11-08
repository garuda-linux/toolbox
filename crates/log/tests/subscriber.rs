//! The installed subscriber end to end, in a process of its own: the one
//! global subscriber and `log` logger are set once here. The events name a
//! target of ours, the test crate's own counts as a dependency.

use std::fs;

use toolbox_log::subscriber;
use tracing::level_filters::LevelFilter;

#[test]
fn tracing_events_and_log_records_reach_the_file_in_one_format() {
    let dir = tempfile::tempdir().unwrap();

    subscriber::init(LevelFilter::INFO, None, None);
    let path = toolbox_log::open_file(Some(dir.path()), "garuda-toolbox-test").unwrap();

    {
        let _apply = tracing::info_span!(target: "toolbox_test", "apply", steps = 2_u32).entered();
        tracing::info!(target: "toolbox_test", program = "pacman", "step started");
    }

    log::warn!(target: "wgpu_core::device", "device lost");
    log::info!(target: "wgpu_core::device", "below the dependency cap");
    tracing::debug!(target: "toolbox_test", "below the level");

    subscriber::set_level(LevelFilter::DEBUG);
    tracing::debug!(target: "toolbox_test", "after the change");

    let text = fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text
        .lines()
        .map(|line| line.split_once(' ').map_or(line, |(_time, rest)| rest))
        .collect();

    assert!(
        lines
            .first()
            .is_some_and(|line| line.ends_with("started (log level info)")),
        "{text}"
    );
    assert!(
        lines.get(2).is_some_and(
            |line| line.starts_with("INFO  toolbox_test: apply{steps=2}: close elapsed=")
        ),
        "{text}"
    );
    assert_eq!(
        [lines.get(1), lines.get(3), lines.get(4), lines.get(5)],
        [
            Some(&"INFO  toolbox_test: apply{steps=2}: step started program=pacman"),
            Some(&"WARN  wgpu_core::device: device lost"),
            Some(&"DEBUG toolbox_test: after the change"),
            None,
        ],
        "{text}"
    );
}
