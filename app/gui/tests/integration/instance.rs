//! A later start's wish reaching the running app: the page it names, and
//! the window asked to the front.

use egui::{UserAttentionType, ViewportCommand};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::command::{self, AppCommand, CommandBus, CommandSender};
use toolbox_gui::instance::Arrival;
use toolbox_gui::page::PageId;
use toolbox_gui::{App, Setup};

use crate::common::launch;

fn app() -> (Harness<'static, App>, CommandSender) {
    let (sender, receiver) = command::channel();
    let commands = sender.clone();
    let harness = launch(Setup {
        commands: Some(CommandBus { sender, receiver }),
        ..Setup::default()
    });

    (harness, commands)
}

fn window_commands(harness: &Harness<'static, App>) -> Vec<ViewportCommand> {
    harness
        .output()
        .viewport_output
        .values()
        .flat_map(|viewport| viewport.commands.clone())
        .collect()
}

#[test]
fn an_arrival_opens_its_page_and_asks_for_the_front() {
    let (mut harness, commands) = app();

    commands.send(AppCommand::Activated(Arrival {
        page: Some(PageId::Maintenance),
        action: None,
        raised: false,
    }));
    harness.step();

    let asked = window_commands(&harness);
    assert!(asked.contains(&ViewportCommand::Minimized(false)));
    assert!(asked.contains(&ViewportCommand::Focus));
    assert!(asked.contains(&ViewportCommand::RequestUserAttention(
        UserAttentionType::Informational
    )));

    harness.run();
    assert!(
        harness
            .query_by_label_contains("Keep the system in a good state")
            .is_some()
    );
}

#[test]
fn a_window_raised_with_a_token_asks_for_nothing_more() {
    let (mut harness, commands) = app();

    commands.send(AppCommand::Activated(Arrival {
        page: None,
        action: None,
        raised: true,
    }));
    harness.step();

    let asked = window_commands(&harness);
    assert!(asked.contains(&ViewportCommand::Minimized(false)));
    assert!(!asked.contains(&ViewportCommand::Focus));
}
