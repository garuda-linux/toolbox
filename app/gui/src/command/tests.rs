use std::collections::BTreeSet;

use egui::{Event, InputState, Key, KeyboardShortcut, Modifiers};

use super::shortcuts::consume;
use super::{AppCommand, Command, ConsoleCommand, UiCommand, channel};
use crate::page::PageId;

#[test]
fn the_first_nine_pages_have_a_digit_and_the_rest_none() {
    assert_eq!(
        UiCommand::Navigate(PageId::Home).shortcut(),
        Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::Num1))
    );
    assert_eq!(
        UiCommand::Navigate(PageId::Kernels).shortcut(),
        Some(KeyboardShortcut::new(Modifiers::COMMAND, Key::Num9))
    );
    assert_eq!(UiCommand::Navigate(PageId::About).shortcut(), None);
}

#[test]
fn no_two_commands_share_a_shortcut() {
    let commands = PageId::ALL
        .into_iter()
        .map(UiCommand::Navigate)
        .chain(UiCommand::ACTIONS);

    let shortcuts: Vec<String> = commands
        .filter_map(UiCommand::shortcut)
        .map(|shortcut| format!("{shortcut:?}"))
        .collect();

    let distinct: BTreeSet<&String> = shortcuts.iter().collect();

    assert_eq!(distinct.len(), shortcuts.len());
}

#[test]
fn commands_arrive_in_the_order_sent() {
    let (sender, receiver) = channel();
    sender.send(UiCommand::Navigate(PageId::Kernels));
    sender.send(AppCommand::CancelPlan);

    let commands: Vec<Command> = receiver.drain().map(|sent| sent.command).collect();

    assert_eq!(
        commands,
        [
            Command::Ui(UiCommand::Navigate(PageId::Kernels)),
            Command::App(AppCommand::CancelPlan),
        ]
    );
}

#[test]
fn every_clone_of_the_sender_reaches_the_same_receiver() {
    let (sender, receiver) = channel();
    let page_sender = sender.clone();
    page_sender.send(UiCommand::ToggleSidebar);
    sender.send(UiCommand::ToggleSidebar);

    assert_eq!(receiver.drain().count(), 2);
}

#[test]
fn sending_after_the_app_is_gone_does_nothing() {
    let (sender, receiver) = channel();
    drop(receiver);

    sender.send(UiCommand::ToggleSidebar);
}

fn pressing(modifiers: Modifiers, key: Key) -> InputState {
    let mut input = InputState::default();
    input.modifiers = modifiers;
    input.events.push(Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    });
    input
}

#[test]
fn a_text_field_keeps_the_keys_that_edit_text() {
    let bound = || {
        vec![
            (KeyboardShortcut::new(Modifiers::NONE, Key::Space), "play"),
            (KeyboardShortcut::new(Modifiers::COMMAND, Key::Z), "undo"),
        ]
    };

    let mut space = pressing(Modifiers::NONE, Key::Space);
    let mut undo = pressing(Modifiers::COMMAND, Key::Z);
    assert_eq!(consume(&mut space, true, bound()), [""; 0]);
    assert_eq!(consume(&mut undo, true, bound()), [""; 0]);

    let mut unfocused = pressing(Modifiers::NONE, Key::Space);
    assert_eq!(consume(&mut unfocused, false, bound()), ["play"]);
}

#[test]
fn the_app_shortcuts_still_work_in_a_text_field() {
    let mut input = pressing(Modifiers::COMMAND, Key::Num9);

    assert_eq!(
        UiCommand::pressed(&mut input, true),
        [UiCommand::Navigate(PageId::Kernels)]
    );
}

#[test]
fn the_shortcut_with_the_most_modifiers_takes_the_press() {
    let bound = || {
        vec![
            (KeyboardShortcut::new(Modifiers::COMMAND, Key::S), "save"),
            (
                KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::S),
                "save as",
            ),
        ]
    };

    let mut both = pressing(Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
    let mut one = pressing(Modifiers::COMMAND, Key::S);

    assert_eq!(consume(&mut both, false, bound()), ["save as"]);
    assert_eq!(consume(&mut one, false, bound()), ["save"]);
}

#[test]
fn only_removing_every_change_is_destructive() {
    let commands = PageId::ALL
        .into_iter()
        .map(UiCommand::Navigate)
        .chain(UiCommand::ACTIONS)
        .chain([UiCommand::ReviewChanges, UiCommand::ClearChanges]);

    let destructive: Vec<UiCommand> = commands
        .filter(|command| command.is_destructive())
        .collect();

    assert_eq!(destructive, [UiCommand::ClearChanges]);
}

#[test]
fn every_command_has_a_tooltip_in_every_language() {
    let ctx = egui::Context::default();
    let commands: Vec<UiCommand> = PageId::ALL
        .into_iter()
        .map(UiCommand::Navigate)
        .chain(UiCommand::ACTIONS)
        .chain([UiCommand::ReviewChanges, UiCommand::ClearChanges])
        .collect();

    for language in crate::i18n::available() {
        crate::i18n::set_language(&ctx, &language);
        for command in &commands {
            let tooltip = crate::i18n::text(&ctx, command.tooltip());
            assert_ne!(tooltip, command.tooltip(), "{command:?} in {language}");
        }
    }
}

#[test]
fn a_sent_command_knows_the_line_that_sent_it() {
    let (sender, receiver) = channel();

    let line = line!() + 1;
    sender.send(UiCommand::ToggleSidebar);

    let sent = receiver.drain().next().unwrap();
    assert_eq!(sent.from.line(), line);
    assert!(sent.from.file().ends_with("command/tests.rs"));
}

#[test]
fn console_input_is_logged_by_its_length_alone() {
    let typed = Command::App(AppCommand::ConsoleInput(b"hunter2\r".to_vec()));

    assert_eq!(typed.for_log(), "ConsoleInput(8 bytes)");
    assert_eq!(Command::Ui(UiCommand::Apply).for_log(), "Ui(Apply)");
}

#[test]
fn shared_output_is_logged_by_its_length_alone() {
    let shared = Command::from(ConsoleCommand::Share("token=s3cret\n".to_owned()));

    assert_eq!(shared.for_log(), "Console(Share(13 bytes))");
}

#[test]
fn ctrl_f_focuses_the_search_also_from_a_text_field() {
    let mut input = pressing(Modifiers::COMMAND, Key::F);

    assert_eq!(
        UiCommand::pressed(&mut input, true),
        [UiCommand::FocusSearch]
    );
}

#[test]
fn f1_alone_opens_the_guide_on_the_open_page() {
    let mut input = pressing(Modifiers::NONE, Key::F1);

    assert_eq!(UiCommand::pressed(&mut input, false), [UiCommand::Help]);
}
