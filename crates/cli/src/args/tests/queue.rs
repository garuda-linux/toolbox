use super::super::{Command, Group, QueueCommand, ToggleCommand};
use super::parse;

fn queue(argv: &[&str]) -> QueueCommand {
    let mut words = vec!["queue"];
    words.extend_from_slice(argv);
    match parse(&words).unwrap().command {
        Some(Command::Queue(Group(command))) => command,
        other => panic!("not a queue command: {other:?}"),
    }
}

#[test]
fn queue_add_takes_a_change_typed_as_its_own_command() {
    let QueueCommand::Add(add) = queue(&["add", "toggle", "on", "pipewire-support"]) else {
        panic!("not queue add");
    };

    let Command::Toggle(ToggleCommand::On(entry)) = add.command().unwrap() else {
        panic!("not toggle on");
    };

    assert_eq!(entry.id, "pipewire-support");
}

#[test]
fn queue_add_hands_the_change_s_own_flags_over() {
    let QueueCommand::Add(add) = queue(&["add", "services", "mask", "foo.service", "--user"])
    else {
        panic!("not queue add");
    };

    let Command::Services(services) = add.command().unwrap() else {
        panic!("not services");
    };

    assert!(services.change().is_ok_and(|(_change, unit)| unit.user));
}

#[test]
fn queue_add_refuses_what_no_command_takes() {
    parse(&["queue", "add"]).unwrap_err();

    let QueueCommand::Add(add) = queue(&["add", "toggle", "on", "not an id"]) else {
        panic!("not queue add");
    };

    add.command().unwrap_err();

    let QueueCommand::Add(window) = queue(&["add", "--page", "maintenance"]) else {
        panic!("not queue add");
    };

    window.command().unwrap_err();
}

#[test]
fn queue_remove_takes_a_number_or_a_name() {
    let QueueCommand::Remove(remove) = queue(&["remove", "2"]) else {
        panic!("not queue remove");
    };

    assert_eq!(remove.change, "2");

    parse(&["queue", "remove", "a\u{7}b"]).unwrap_err();
}

#[test]
fn queue_apply_asks_like_every_change() {
    let QueueCommand::Apply(confirm) = queue(&["apply", "--dry-run"]) else {
        panic!("not queue apply");
    };

    assert!(confirm.dry_run);

    parse(&["queue", "apply", "--yes", "--dry-run"]).unwrap_err();
    assert!(
        parse(&["queue", "apply", "--yes"])
            .unwrap()
            .command
            .unwrap()
            .confirm()
            .is_some_and(|asked| asked.yes)
    );
}

#[test]
fn queue_clear_asks_unless_yes() {
    let QueueCommand::Clear(clear) = queue(&["clear", "--yes"]) else {
        panic!("not queue clear");
    };

    assert!(clear.yes);
}
