use std::time::Duration;

use clap::error::ErrorKind;

use super::super::Command;
use super::super::tests::parse;
use super::{Interval, StatusArgs};

fn status(args: &[&str]) -> StatusArgs {
    match parse(args).unwrap().command {
        Some(Command::Status(status)) => status,
        other => panic!("not status: {other:?}"),
    }
}

#[test]
fn a_watch_looks_again_every_minute_unless_told_otherwise() {
    let once = status(&["status"]);
    assert!(!once.watch);

    let watch = status(&["status", "--watch"]);
    assert!(watch.watch);
    assert_eq!(watch.interval.duration(), Duration::from_mins(1));
}

#[test]
fn the_interval_takes_a_duration_with_its_unit() {
    let every = |text: &str| status(&["status", "--watch", "--interval", text]).interval;

    assert_eq!(every("30s").duration(), Duration::from_secs(30));
    assert_eq!(every("5m").duration(), Duration::from_mins(5));
    assert_eq!(every("1h30m").duration(), Duration::from_mins(90));
}

#[test]
fn an_interval_without_a_unit_or_under_a_second_is_refused() {
    for text in ["30", "500ms", "-5m", "soon"] {
        let given = format!("--interval={text}");
        let error = parse(&["status", "--watch", &given]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ValueValidation, "{text}");
    }
}

#[test]
fn an_interval_needs_a_watch() {
    let error = parse(&["status", "--interval", "5m"]).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
}

#[test]
fn a_watch_takes_brief_and_json() {
    let both = status(&["status", "--watch", "--brief", "--json"]);

    assert!(both.watch && both.brief && both.read.json);
    assert_eq!(Interval::default().duration(), Duration::from_mins(1));
}
