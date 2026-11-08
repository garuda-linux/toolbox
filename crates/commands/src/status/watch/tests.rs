use std::collections::VecDeque;

use toolbox_cli::Exit;
use toolbox_cli::output::{Paint, Style};

use super::{Shape, Watch, watch};
use crate::status::{Package, Part, StatusView};

fn view(updates: usize) -> StatusView {
    let packages = (0..updates)
        .map(|at| Package {
            name: format!("package-{at}"),
            from: "1".to_owned(),
            to: "2".to_owned(),
        })
        .collect();

    StatusView {
        updates: Part::Ok { value: packages },
        aur_updates: Part::Unavailable,
        files_to_merge: Part::Ok { value: Vec::new() },
        last_update_days: Part::Ok { value: Some(0) },
        health: Part::Ok { value: Vec::new() },
        restart_needed: Part::Ok { value: false },
        facts: None,
    }
}

fn text() -> Shape {
    Shape::Text {
        paint: Paint::new(Style::Plain),
        verbose: false,
    }
}

#[test]
fn the_facts_show_under_the_time_and_again_only_once_they_changed() {
    let mut watching = Watch::new(text());

    let first = watching.next(view(0), "14:02").unwrap();
    assert!(first.starts_with("14:02\nStatus\n"), "{first}");

    assert_eq!(watching.next(view(0), "14:03"), None);

    let changed = watching.next(view(2), "14:04").unwrap();
    assert!(changed.starts_with("\n14:04\nStatus\n"), "{changed}");
    assert!(changed.contains("2 packages"), "{changed}");
}

#[test]
fn brief_is_one_line_behind_the_time() {
    let mut watching = Watch::new(Shape::Brief);

    assert_eq!(
        watching.next(view(0), "14:02").as_deref(),
        Some("14:02  all good\n")
    );
    assert_eq!(
        watching.next(view(1), "14:03").as_deref(),
        Some("14:03  1 update\n")
    );
}

/// JSON Lines: one object per change, the shape `status --json` has.
#[test]
fn json_is_one_line_per_change() {
    let mut watching = Watch::new(Shape::Json);

    let line = watching.next(view(0), "14:02").unwrap();

    assert!(line.starts_with("{\"schema\":1,\"updates\":"), "{line}");
    assert!(
        line.ends_with("}\n") && line.matches('\n').count() == 1,
        "{line}"
    );
    assert_eq!(watching.next(view(0), "14:03"), None);
}

/// Ctrl+C is the way to end a watch, so it ends with 0, as SIGTERM and
/// SIGHUP do.
#[test]
fn a_watch_reads_until_ctrl_c_and_ends_with_0() {
    let mut reads = VecDeque::from([view(0), view(0), view(3)]);
    let mut waits = VecDeque::from([false, false, true]);
    let mut out = Vec::new();

    let exit = watch(
        Watch::new(Shape::Brief),
        || reads.pop_front().unwrap(),
        || waits.pop_front().unwrap(),
        || "09:00".to_owned(),
        &mut out,
    );

    assert_eq!(exit, Exit::Success);
    assert!(reads.is_empty());
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "09:00  all good\n09:00  3 updates\n"
    );
}
