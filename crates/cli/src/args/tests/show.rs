use super::super::{Command, ServicesRead, ToggleCommand};
use super::parse;

#[test]
fn toggle_show_takes_an_id_and_json() {
    let Some(Command::Toggle(ToggleCommand::Show(show))) =
        parse(&["toggle", "show", "pipewire-support", "--json"])
            .unwrap()
            .command
    else {
        panic!("not toggle show");
    };

    assert_eq!(show.id, "pipewire-support");
    assert!(show.read.json);
}

#[test]
fn services_show_takes_a_unit_as_systemctl_does() {
    let Some(Command::Services(services)) = parse(&["services", "show", "bluetooth", "--user"])
        .unwrap()
        .command
    else {
        panic!("not services");
    };

    let Err(ServicesRead::Show(show)) = services.change() else {
        panic!("not services show");
    };

    assert_eq!(show.unit.as_str(), "bluetooth.service");
    assert!(show.user);
    assert!(!show.read.json);
}

#[test]
fn a_show_changes_nothing() {
    for words in [
        &["toggle", "show", "docker"][..],
        &["services", "show", "sshd.service"],
    ] {
        let command = parse(words).unwrap().command.unwrap();
        assert!(command.confirm().is_none(), "{words:?}");
    }
}
