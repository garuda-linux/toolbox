//! An install that met a mirror without the file it needs, with pacman's
//! own output.

use egui_kittest::kittest::Queryable as _;
use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Failure, Outcome};

use super::{app, script, unfocus, wait_for};

/// pacman's install of cowsay from a mirror that answers 404, as it
/// printed it.
const PACMAN_404: &str = include_str!(
    "../../../../../crates/core/src/features/packages/mirror/fixtures/pacman-404.stdout"
);

/// The install's step failed on a file its mirror no longer has, with
/// pacman's exit code.
fn missing_file_script() -> String {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Output {
            step,
            data: PACMAN_404.to_owned(),
        },
        Event::Finished {
            step,
            outcome: Outcome::Failed {
                cause: Failure::ExitCode { code: 1_i32 },
                cleanup_failed: Vec::new(),
            },
        },
        Event::Done,
    ];

    format!("{}; exit 1", script(&events))
}

#[test]
fn an_install_that_met_a_missing_mirror_file_offers_an_update_first() {
    let (mut harness, _runtime) = app(missing_file_script(), &["cowsay"]);

    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "Update the system and try again");
    unfocus(&mut harness);
    harness.snapshot("console/mirror_missing_dark");

    harness.get_by_label("Console output actions").click();
    harness.run();
    harness.get_by_label("Copy output").click();
    harness.step();
    let quoted = "failed retrieving file 'cowsay-3.8.4-1-any.pkg.tar.zst' from 127.0.0.1:8765";
    let mut commands = harness.output().platform_output.commands.iter();
    let copied = commands.any(
        |command| matches!(command, egui::OutputCommand::CopyText(text) if text.contains(quoted)),
    );

    assert!(copied, "pacman's quotes did not pass through");

    harness
        .get_by_label("Update the system and try again")
        .click();
    harness.step();

    wait_for(&mut harness, "pacman -Syu --noconfirm");
    assert!(
        harness
            .query_by_label_contains("pacman -S --needed --noconfirm -- cowsay")
            .is_some()
    );
}
