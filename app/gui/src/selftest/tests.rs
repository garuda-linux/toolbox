use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use egui::{Key, TextEdit};
use egui_kittest::Harness;
use tempfile::TempDir;

use super::{RESULT_FILE, Runner, SUITES, Step, requested, steps};

struct Playground {
    pressed: bool,
    typed: String,
    shortcuts: usize,
}

fn playground(steps: Vec<Step>, dir: &Path, patience: Duration) -> Harness<'static, Playground> {
    let state = Playground {
        pressed: false,
        typed: String::new(),
        shortcuts: 0,
    };

    let harness = toolbox_gui_kit::testing::harness(400.0, 300.0).build_ui_state(
        |ui, state: &mut Playground| {
            ui.label("Hello");
            if ui.button("Press me").clicked() {
                state.pressed = true;
            }

            if state.pressed {
                ui.label("Pressed");
            }

            let label = ui.label("Field");
            ui.add(TextEdit::singleline(&mut state.typed))
                .labelled_by(label.id);
            ui.label(format!("Typed: {}", state.typed));

            if ui.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, Key::P)) {
                state.shortcuts += 1;
            }

            ui.label(format!("Shortcuts: {}", state.shortcuts));
        },
        state,
    );

    harness
        .ctx
        .add_plugin(Runner::new(steps, dir.to_path_buf(), patience));
    harness
}

fn verdict(harness: &mut Harness<'static, Playground>, dir: &Path) -> String {
    let start = Instant::now();
    while !dir.join(RESULT_FILE).exists() {
        assert!(start.elapsed() < Duration::from_secs(20), "no verdict");
        harness.step();
    }

    fs::read_to_string(dir.join(RESULT_FILE)).unwrap()
}

#[test]
fn a_suite_finds_clicks_types_and_presses_keys() {
    let dir = TempDir::new().unwrap();
    let [field, text] = Step::type_into("Field", "abc");
    let suite = vec![
        Step::wait("Hello"),
        Step::click("Press me"),
        Step::wait("Pressed"),
        field,
        text,
        Step::wait("Typed: abc"),
        Step::shortcut(Key::P),
        Step::wait("Shortcuts: 1"),
        Step::WaitGone("Not there"),
        Step::WaitContaining("Typed:"),
    ];

    let mut harness = playground(suite, dir.path(), Duration::from_secs(10));

    assert_eq!(verdict(&mut harness, dir.path()), "PASS\n");
}

#[test]
fn either_of_two_labels_ends_the_wait() {
    let dir = TempDir::new().unwrap();
    let suite = vec![
        Step::WaitEither("Never", "Hello"),
        Step::WaitEither("Hello", "Never"),
    ];

    let mut harness = playground(suite, dir.path(), Duration::from_secs(10));

    assert_eq!(verdict(&mut harness, dir.path()), "PASS\n");
}

#[test]
fn a_step_that_never_happens_fails_with_its_number() {
    let dir = TempDir::new().unwrap();
    let suite = vec![Step::wait("Hello"), Step::wait("Never")];
    let mut harness = playground(suite, dir.path(), Duration::from_millis(50));

    let verdict = verdict(&mut harness, dir.path());

    assert!(verdict.starts_with("FAIL step 2 "), "{verdict}");
    assert!(verdict.contains("timed out"), "{verdict}");
}

#[test]
fn a_failed_step_leaves_a_shot_of_the_window() {
    let dir = TempDir::new().unwrap();
    let suite = vec![Step::wait("Hello"), Step::wait("Never")];
    let mut harness = playground(suite, dir.path(), Duration::from_secs(1));

    let verdict = verdict(&mut harness, dir.path());

    assert!(verdict.starts_with("FAIL step 2 "), "{verdict}");
    assert!(dir.path().join("failed.png").exists());
}

#[test]
fn the_environment_names_folder_and_suite() {
    let lookup = |pairs: &'static [(&str, &str)]| {
        move |key: &str| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| OsString::from(value))
        }
    };

    assert_eq!(requested(lookup(&[])), None);
    assert_eq!(
        requested(lookup(&[("GARUDA_TOOLBOX_SELFTEST_DIR", "/tmp/shots")])),
        Some(("/tmp/shots".into(), "all".to_owned()))
    );
    assert_eq!(
        requested(lookup(&[
            ("GARUDA_TOOLBOX_SELFTEST_DIR", "/tmp/shots"),
            ("GARUDA_TOOLBOX_SELFTEST_SUITE", "components"),
        ])),
        Some(("/tmp/shots".into(), "components".to_owned()))
    );
}

#[test]
fn all_runs_every_suite_and_unknown_suites_are_none() {
    let each: usize = SUITES
        .iter()
        .map(|(name, _)| steps(name).unwrap().len())
        .sum();

    assert_eq!(steps("all").unwrap().len(), each);
    assert_eq!(steps("no-such-suite"), None);
}

#[test]
fn a_shot_waits_out_every_slide_and_fade() {
    use toolbox_gui_kit::{motion, theme};

    assert!(!super::runner::settled(Duration::ZERO));
    for seconds in [
        motion::SEGMENT_SLIDE,
        motion::SHEET_ENTER,
        motion::SHEET_EXIT,
        motion::TOAST_ENTER,
        motion::POPOVER_ENTER,
        motion::DIM_FADE,
        theme::ANIMATION_TIME,
    ] {
        let after = Duration::from_secs_f32(seconds);
        assert!(
            !super::runner::settled(after),
            "{seconds} s is still moving"
        );
    }
}

/// The detail steps need pacman's databases, which only the Arch runner
/// has: their suite runs alone and `all` leaves it out.
#[test]
fn the_arch_suite_runs_alone_and_all_leaves_it_out() {
    let arch = steps("packages-arch").unwrap();

    assert!(arch.contains(&Step::wait("Back to pacman")));
    assert!(
        !steps("all")
            .unwrap()
            .contains(&Step::wait("Back to pacman"))
    );
}

/// The AUR runs offline on the dry run's sample list, so `all` holds it.
/// Its install question ends in Cancel: nothing is queued.
#[test]
fn the_aur_suite_runs_in_all_and_queues_nothing() {
    let aur = steps("packages-aur").unwrap();

    assert!(aur.contains(&Step::button("Cancel")));
    assert!(!aur.contains(&Step::button("Install")));
    assert!(
        steps("all")
            .unwrap()
            .contains(&Step::Shot("packages-aur-detail"))
    );
}
