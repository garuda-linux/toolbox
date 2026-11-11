use super::super::{Command, ServicesCommand};
use super::parse;

fn unit_of(words: &[&str]) -> String {
    let Some(Command::Services(ServicesCommand::Start(args))) = parse(words).unwrap().command
    else {
        panic!("not services start");
    };

    args.unit.as_str().to_owned()
}

#[test]
fn a_unit_without_a_type_is_a_service_as_for_systemctl() {
    assert_eq!(
        unit_of(&["services", "start", "bluetooth"]),
        "bluetooth.service"
    );
    assert_eq!(
        unit_of(&["services", "start", "getty@tty2"]),
        "getty@tty2.service"
    );
}

#[test]
fn a_unit_with_its_type_stays_as_typed() {
    assert_eq!(
        unit_of(&["services", "start", "docker.socket"]),
        "docker.socket"
    );
    assert_eq!(
        unit_of(&["services", "start", "sshd.service"]),
        "sshd.service"
    );
}

#[test]
fn a_bad_unit_is_refused_by_the_name_typed() {
    let error = parse(&["services", "start", "a/b"])
        .unwrap_err()
        .to_string();

    assert!(error.contains("`a/b`"), "{error}");
    assert!(!error.contains("a/b.service"), "{error}");
}
