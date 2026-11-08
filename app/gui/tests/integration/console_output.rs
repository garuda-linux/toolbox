//! The Console's menu for its output: copy it, share it as an encrypted
//! paste through the dry-run paste server, and clear it.

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tokio::runtime::Runtime;
use toolbox_gui::diagnostics::DryRunTransport;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, PreferencesSetup, Setup, apply, share};

use crate::common::{HEIGHT, installing, launch, window};
use crate::console::{FakeLauncher, JAVA, silent_script, succeeding_script, wait_for};

/// The output's menu, named for screen readers.
const MENU: &str = "Console output actions";

/// The line an idle terminal shows in place of output.
const IDLE_HINT: &str = "The output of the next apply shows here.";

/// java-runtime applied with the fake helper, so the Console has output.
fn applied() -> (Harness<'static, App>, Runtime) {
    applied_with(succeeding_script(), false)
}

/// java-runtime applied, the fake helper running `script`, an upload's link
/// copied when `copy`.
fn applied_with(script: String, copy: bool) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let changes = installing(JAVA);

    let mut harness = window(HEIGHT).build_eframe(move |cc| {
        let launcher = Arc::new(FakeLauncher(script));
        let mut preferences = PreferencesSetup::default();
        preferences.settings.diagnostics.copy_to_clipboard = copy;
        let setup = Setup {
            snapshot: Some(crate::console::installed_after_the_run(
                &handle,
                &cc.egui_ctx,
                JAVA,
            )),
            queue: Queue::in_memory(changes),
            apply: Some(apply::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                launcher,
            )),
            output_share: Some(share::spawn(
                &handle,
                toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                Arc::new(DryRunTransport),
            )),
            preferences,
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    harness.run();
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "All changes applied");
    (harness, runtime)
}

fn choose(harness: &mut Harness<'static, App>, item: &str) {
    harness.get_by_label(MENU).click();
    harness.run();
    harness.get_by_label(item).click();
    harness.step();
}

#[test]
fn copy_puts_the_output_on_the_clipboard_and_says_so() {
    let (mut harness, _runtime) = applied();

    choose(&mut harness, "Copy output");

    let copied = harness.output().platform_output.commands.iter().any(|command| {
        matches!(command, egui::OutputCommand::CopyText(text) if text.contains("Proceed with installation?"))
    });

    assert!(copied, "the output did not reach the clipboard");
    harness.run();
    assert!(
        harness
            .query_by_label("Copied the output to the clipboard.")
            .is_some()
    );
}

#[test]
fn share_asks_with_the_output_then_shows_the_link() {
    let (mut harness, _runtime) = applied();

    choose(&mut harness, "Share output…");
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Upload this output?")
            .is_some()
    );
    assert!(
        harness
            .get_by_label("The text that will be uploaded")
            .value()
            .is_some_and(|text| text.contains("Proceed with installation?")),
        "the question does not show what goes out"
    );

    harness.get_by_label("Upload").click();
    harness.step();
    wait_for(&mut harness, "Copy link");
}

#[test]
fn clear_empties_the_output_and_its_menu_goes() {
    let (mut harness, _runtime) = applied();

    choose(&mut harness, "Clear output");
    harness.run();

    assert!(harness.query_by_label(MENU).is_none());
}

/// The row after the upload hides the link, whose key changes each time.
#[test]
fn snapshots_of_the_menu_the_question_and_the_link_row() {
    let (mut harness, _runtime) = applied();
    crate::system::step_until(&mut harness, "the queue to empty", |harness| {
        harness.query_by_label("Review").is_none()
    });

    harness.get_by_label(MENU).click();
    harness.run();
    harness.snapshot("console/menu_dark");

    harness.get_by_label("Share output…").click();
    harness.run();
    harness.snapshot("console/share_question_dark");

    harness.get_by_label("Upload").click();
    harness.step();
    wait_for(&mut harness, "Copy link");
    harness.snapshot("console/shared_dark");
}

fn shared(harness: &mut Harness<'static, App>) {
    choose(harness, "Share output…");
    harness.run();
    harness.get_by_label("Upload").click();
    harness.step();
    wait_for(harness, "Copy link");
}

#[test]
fn the_link_takes_one_row_and_the_terminal_keeps_its_room() {
    let (mut harness, _runtime) = applied();
    let before = harness.get_by_label("Console output").rect().height();

    shared(&mut harness);

    let after = harness.get_by_label("Console output").rect().height();
    let row = harness.get_by_label("Copy link").rect().height();
    let spacing = harness.ctx.global_style().spacing.item_spacing.y;
    let gap = toolbox_gui_kit::tokens::current(&harness.ctx)
        .sizes
        .control_gap;

    let gaps = spacing.mul_add(2.0, gap);

    assert!(
        before - after <= row + gaps,
        "the terminal went from {before} to {after}, a row is {row}"
    );
    assert!(
        harness.query_by_label("Delete now…").is_none(),
        "the delete link sits in the row"
    );
}

#[test]
fn with_the_setting_on_the_link_goes_to_the_clipboard() {
    let (mut harness, _runtime) = applied_with(succeeding_script(), true);

    shared(&mut harness);
    harness.run();

    assert!(
        harness
            .query_by_label("Copied the link to the clipboard.")
            .is_some()
    );
}

#[test]
fn clear_empties_the_steps_too_and_the_console_is_idle() {
    let (mut harness, _runtime) = applied();

    choose(&mut harness, "Clear output");
    harness.run();

    assert!(harness.query_by_label("Done").is_none(), "the step stayed");
    assert!(harness.query_by_label("All changes applied").is_none());
    assert!(
        harness
            .query_by_label("No changes are being applied")
            .is_some()
    );
}

#[test]
fn an_idle_terminal_says_where_the_output_will_show() {
    let mut harness = launch(Setup::default());

    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::J);
    harness.run();

    assert!(harness.query_by_label(IDLE_HINT).is_some());
}

#[test]
fn output_takes_the_place_of_the_idle_hint() {
    let (harness, _runtime) = applied();

    assert!(harness.query_by_label(IDLE_HINT).is_none());
}

#[test]
fn a_plan_that_printed_nothing_can_be_cleared_but_not_copied() {
    let (mut harness, _runtime) = applied_with(silent_script(), false);
    harness.get_by_label(MENU).click();
    harness.run();

    let copy = harness.get_by_label("Copy output");
    assert!(
        copy.accesskit_node().is_disabled(),
        "copy offered without output"
    );

    harness.get_by_label("Clear output").click();
    harness.run();
    assert!(
        harness
            .query_by_label("No changes are being applied")
            .is_some()
    );
}

#[test]
fn another_page_after_a_finished_run_puts_the_console_away() {
    let (mut harness, _runtime) = applied();
    assert!(harness.query_by_label("All changes applied").is_some());

    harness.get_by_role_and_label(Role::Tab, "Kernels").click();
    harness.run();

    assert!(
        harness.query_by_label("All changes applied").is_none(),
        "the finished Console still covers the next page"
    );
}

#[test]
fn a_run_that_printed_nothing_says_so_in_the_terminal() {
    let (harness, _runtime) = applied_with(silent_script(), false);

    assert!(harness.query_by_label(IDLE_HINT).is_none());
    assert!(
        harness
            .query_by_label("The changes printed nothing.")
            .is_some()
    );
}
