//! A later start's wish reaching the running app: the page it names, and
//! the window asked to the front.

use std::sync::Arc;

use egui::{UserAttentionType, ViewportCommand};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::sync::oneshot;
use toolbox_core::ChangeSet;
use toolbox_gui::command::{self, AppCommand, CommandBus, CommandSender};
use toolbox_gui::instance::{Arrival, Arrivals};
use toolbox_gui::page::PageId;
use toolbox_gui::queue::edit::QueueEdit;
use toolbox_gui::{App, Setup};
use toolbox_instance::bus::queue::{Ask, Desk as _, Question, Refusal};

use crate::common::{installing, launch};

const EDITED: &str = "The queue was changed from the command line.";

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

/// The app with a desk for queue commands, as the bus has.
fn with_desk() -> (Harness<'static, App>, Arc<Arrivals>) {
    let arrivals = Arc::new(Arrivals::new(command::channel().0));
    let harness = launch(Setup {
        arrivals: Some(Arc::clone(&arrivals)),
        ..Setup::default()
    });

    (harness, arrivals)
}

fn ask(arrivals: &Arrivals, question: Question) -> oneshot::Receiver<Result<String, Refusal>> {
    let (answer, answered) = oneshot::channel();
    arrivals.ask(Ask { question, answer });
    answered
}

#[test]
fn an_edit_from_a_command_shows_a_note() {
    let (mut harness, arrivals) = with_desk();
    let edit = QueueEdit::between(&ChangeSet::default(), &installing(&["firefox"]));

    let mut answered = ask(&arrivals, Question::Edit(edit.to_text()));
    harness.run();

    assert_eq!(answered.try_recv(), Ok(Ok(String::new())));
    assert!(harness.query_by_label(EDITED).is_some());
}

#[test]
fn a_command_that_only_reads_the_queue_shows_no_note() {
    let (mut harness, arrivals) = with_desk();

    let mut answered = ask(&arrivals, Question::Changes);
    harness.run();

    assert!(matches!(answered.try_recv(), Ok(Ok(_))));
    assert!(harness.query_by_label(EDITED).is_none());
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
