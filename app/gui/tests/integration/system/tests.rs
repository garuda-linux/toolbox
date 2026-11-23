use egui::{Key, Modifiers};
use egui_kittest::kittest::NodeT as _;

use super::*;

#[test]
fn the_first_read_shows_and_then_drops_what_is_done() {
    let (mut harness, _runtime, answer, _reads) = app();

    until_reading_notice(&mut harness);
    harness.get_by_label("Review").click();
    harness.step();
    step_until(&mut harness, "the popover", |harness| {
        harness.query_by_label("Install firefox").is_some()
    });

    answer.send((installed(&["firefox"]), Vec::new())).unwrap();

    step_until(&mut harness, "firefox leaving the queue", |harness| {
        harness.query_by_label("Install firefox").is_none()
    });
    first_read(&mut harness);
    assert!(harness.query_by_label("Install vlc").is_some());
}

#[test]
fn failed_probes_are_named_in_the_status_bar_which_reads_again() {
    let (mut harness, _runtime, answer, reads) = app();
    let issues = vec![
        ProbeIssue {
            probe: Probe::Units,
            error: ProbeError::fake("no bus"),
        },
        ProbeIssue {
            probe: Probe::Grub,
            error: ProbeError::Unavailable,
        },
        ProbeIssue {
            probe: Probe::Accounts,
            error: ProbeError::fake("no passwd"),
        },
    ];

    answer.send((Snapshot::default(), issues)).unwrap();

    step_until(
        &mut harness,
        "the failure notice",
        shows("Some parts of the system could not be read: system services, users and groups"),
    );

    harness.get_by_label("Read Again").click();
    step_until(&mut harness, "a second read", |_| {
        reads.load(Ordering::SeqCst) == 2
    });
    answer.send((Snapshot::default(), Vec::new())).unwrap();
}

#[test]
fn coming_back_to_the_window_reads_the_system_again() {
    let (mut harness, _runtime, answer, reads) = app();
    answer.send((Snapshot::default(), Vec::new())).unwrap();
    step_until(&mut harness, "the first read", |harness| {
        harness
            .query_all_by_label("Reading the system…")
            .next()
            .is_none()
            && reads.load(Ordering::SeqCst) == 1
    });

    harness.input_mut().focused = false;
    harness.step();
    harness.input_mut().focused = true;
    harness.step();

    step_until(&mut harness, "a second read", |_| {
        reads.load(Ordering::SeqCst) == 2
    });
    answer.send((Snapshot::default(), Vec::new())).unwrap();
}

#[test]
fn a_finished_plan_reads_the_system_and_empties_the_queue() {
    let (mut harness, _runtime, answer, reads) = app();
    answer.send((installed(&[]), Vec::new())).unwrap();
    first_read(&mut harness);

    harness.get_by_label("Apply").click();
    harness.step();
    step_until(&mut harness, "the plan's end", shows("All changes applied"));
    step_until(&mut harness, "a second read", |_| {
        reads.load(Ordering::SeqCst) == 2
    });
    answer
        .send((installed(&["firefox", "vlc"]), Vec::new()))
        .unwrap();

    step_until(&mut harness, "the queue emptying", |harness| {
        harness.query_by_label_contains("queued").is_none()
    });
}

#[test]
fn apply_waits_for_the_first_read() {
    let (mut harness, _runtime, answer, _reads) = app();
    until_reading_notice(&mut harness);

    assert!(harness.get_by_label("Apply").accesskit_node().is_disabled());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.step();
    assert!(harness.query_by_label("Console").is_none());

    answer.send((installed(&[]), Vec::new())).unwrap();
    step_until(&mut harness, "the first read", |harness| {
        !harness.get_by_label("Apply").accesskit_node().is_disabled()
    });

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    step_until(&mut harness, "the plan's end", shows("All changes applied"));
}

/// Fails closed: a read without the kernel packages leaves the kernels'
/// guard waiting, so the restored packages never apply unchecked.
#[test]
fn apply_waits_while_a_guard_lacks_its_part() {
    let (mut harness, _runtime, answer, _reads) = app();
    let mut lacking = Snapshot::default();
    lacking.insert(InstalledPackages(packages(&[])));
    lacking.insert(SyncPackages::default());
    answer.send((lacking, Vec::new())).unwrap();
    first_read(&mut harness);
    harness.run();

    let apply = harness.get_by_label("Apply");
    assert!(apply.accesskit_node().is_disabled());
    assert_eq!(
        apply.accesskit_node().description().as_deref(),
        Some("Apply waits until the system has been read.")
    );

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    harness.step();
    assert!(harness.query_by_label("Console").is_none());
}

/// How far above the window's bottom the side panels end: the status
/// bar's height, or nothing without it.
fn bar_height(harness: &Harness<'static, App>) -> f32 {
    let margin = f32::from(
        toolbox_gui_kit::tokens::Sizes::DEFAULT
            .sidebar_padding
            .bottom,
    );

    let sidebar = harness.get_by_label("About").rect().bottom() + margin;

    harness.ctx.content_rect().bottom() - sidebar
}

#[test]
fn a_read_alone_brings_no_bar_only_a_note_in_the_sidebar() {
    let (mut harness, _runtime, answer, _reads) = app();
    until_reading_notice(&mut harness);
    assert!(bar_height(&harness) < 1.0, "{}", bar_height(&harness));

    answer.send((installed(&[]), Vec::new())).unwrap();
    first_read(&mut harness);
    harness.run();

    assert!(bar_height(&harness) < 1.0, "{}", bar_height(&harness));
}

#[test]
fn a_wait_that_runs_out_leaves_a_screenshot() {
    let screenshot = std::path::Path::new(FAILURES).join("a-label-that-never-shows.new.png");

    // A snapshot run with UPDATE_SNAPSHOTS leaves this picture behind, and
    // the next wait must not take it as a reference that already matches.
    let leftover = std::path::Path::new(FAILURES).join("a-label-that-never-shows.png");
    drop(std::fs::remove_file(&screenshot));
    let mut harness =
        toolbox_gui_kit::testing::harness(200.0, 100.0).build_ui(|ui| drop(ui.label("Waiting")));

    std::fs::create_dir_all(FAILURES).unwrap();
    harness.run();
    harness.render().unwrap().save(&leftover).unwrap();

    let waited = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        step_within(
            &mut harness,
            "a label that never shows",
            |harness| harness.query_by_label("Done").is_some(),
            Duration::from_millis(50),
        );
    }));

    std::fs::remove_file(leftover).unwrap();
    assert!(waited.is_err());
    assert!(screenshot.exists());
    std::fs::remove_file(screenshot).unwrap();
}
