//! Diagnostics with its real service and the dry-run paste server.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::features::diagnostics::{Collector, Outcome};
use toolbox_gui::diagnostics::{self, DryRunTransport};
use toolbox_gui::testing::FixedClock;
use toolbox_gui::{App, PreferencesSetup, Setup};
use toolbox_system::DiagnosticSource;

use crate::common::{Fixture, TALL, window};
use crate::look::look_for;
use crate::system::{shows, step_until};

/// 12:41 UTC, so the report shot keeps its header.
const COLLECTED: i64 = 1_792_327_265_i64;

/// Every collector answers a short coloured report of its own name.
struct Fake;

impl DiagnosticSource for Fake {
    fn collect(&self, collector: Collector, _cancel: &Arc<AtomicBool>) -> Option<Outcome> {
        Some(Outcome::Report(format!(
            "{} says hello\n\u{1b}[33mwarning: something\u{1b}[0m\nlast line\n",
            collector.as_str()
        )))
    }
}

pub fn diagnostics_page(copy_reports: bool) -> Fixture {
    diagnostics_page_on(copy_reports, Theme::Dark)
}

pub fn diagnostics_page_on(copy_reports: bool, theme: Theme) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let look = look_for(&runtime, theme);
    let mut harness = window(TALL).with_theme(theme).build_eframe(move |cc| {
        let mut preferences = PreferencesSetup::default();
        preferences.settings.diagnostics.copy_to_clipboard = copy_reports;

        let link = diagnostics::spawn(
            &handle,
            toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
            Arc::new(Fake),
            Arc::new(DryRunTransport),
            Arc::new(FixedClock(COLLECTED)),
        );

        App::new(
            cc,
            Setup {
                look,
                diagnostics: Some(link),
                preferences,
                ..Setup::default()
            },
        )
    });

    harness.run();
    harness
        .get_by_role_and_label(Role::Tab, "Diagnostics")
        .click();
    harness.run();
    Fixture::new(harness, runtime)
}

pub fn collect(harness: &mut Harness<'static, App>) {
    harness.get_by_label("Collect System information").click();
    step_until(harness, "the report", shows("inxi says hello"));
    harness.run();
}

#[test]
fn a_report_is_collected_and_uploaded_to_the_dry_run_server() {
    let mut fixture = diagnostics_page(false);
    let harness = fixture.harness();

    collect(harness);
    harness.get_by_label("Share Report…").click();
    harness.run();
    harness.get_by_label("Upload").click();
    step_until(harness, "the link", shows("Copy Link"));

    assert!(harness.query_by_label("Uploaded.").is_some());
}

#[test]
fn with_the_setting_on_a_finished_report_is_copied_at_once() {
    let mut fixture = diagnostics_page(true);
    let harness = fixture.harness();

    collect(harness);

    assert!(
        harness
            .query_by_label("Copied the report to the clipboard.")
            .is_some()
    );
}

#[test]
fn the_report_header_stays_inside_its_card_beside_the_sidebar() {
    let mut fixture = diagnostics_page(false);
    let harness = fixture.harness();

    collect(harness);

    let card = harness
        .get_by_role_and_label(Role::Document, "Diagnostics report")
        .rect();

    let window = harness.ctx.content_rect();
    assert!(card.right() < window.right(), "the card {card:?}");

    for (role, name) in [
        (Role::TextInput, "Search the report"),
        (Role::Button, "Copy Report"),
        (Role::Button, "Share Report…"),
    ] {
        let control = harness.get_by_role_and_label(role, name).rect();
        assert!(
            card.contains_rect(control),
            "{name} at {control:?} leaves the card {card:?}"
        );
    }
}
