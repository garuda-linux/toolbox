mod aur_upgrade;
mod dismiss;
mod looked_at;
mod orders;
mod review;
mod runs;

use egui_kittest::Harness;

use super::{App, Setup};
use crate::apply::{PlanId, Report};
use crate::command::{AppCommand, Command, DiscoverCommand, MaintenanceCommand};
use crate::page::PageId;
use crate::page_service::Answer;
use crate::pages::maintenance::Tab;

#[test]
fn a_frame_with_the_same_queue_builds_no_queue_rows() {
    let mut harness = Harness::builder().build_eframe(|cc| App::new(cc, Setup::default()));
    harness.run();
    let built = harness.state().views.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().views.builds(), built);

    harness.state_mut().queue.clear();
    harness.run();
    assert_eq!(harness.state().views.builds(), built + 1);
}

/// An app on Home whose last run finished with the Console still open.
fn after_a_run() -> Harness<'static, App> {
    let mut harness = Harness::builder().build_eframe(|cc| App::new(cc, Setup::default()));
    harness.run();

    let app = harness.state_mut();
    app.state.page = PageId::Home;
    app.state.console_open = true;
    app.console.take(Report::PlanStarted {
        plan: PlanId::new(1),
        steps: Vec::new(),
    });
    app.console.take(Report::PlanFinished {
        plan: PlanId::new(1),
        succeeded: true,
    });
    harness
}

fn send(harness: &mut Harness<'static, App>, command: AppCommand) {
    let ctx = harness.ctx.clone();
    harness.state_mut().run(&ctx, Command::App(command));
}

#[test]
fn a_page_change_a_page_asks_for_closes_a_finished_console() {
    let changes = [
        AppCommand::Discover(DiscoverCommand::Reveal("Firefox".to_owned())),
        AppCommand::Maintenance(MaintenanceCommand::Reveal(Tab::Repairs)),
        AppCommand::Answer(Answer::ShowPage(PageId::Boot)),
    ];

    for change in changes {
        let mut harness = after_a_run();
        send(&mut harness, change.clone());

        assert!(!harness.state().state.console_open, "{change:?}");
    }
}

#[test]
fn a_page_change_a_page_asks_for_skips_a_page_this_system_hides() {
    let mut harness = after_a_run();
    send(
        &mut harness,
        AppCommand::Answer(Answer::ShowPage(PageId::BootRepair)),
    );

    assert_eq!(harness.state().state.page, PageId::Home);
}
