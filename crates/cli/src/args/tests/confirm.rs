use super::super::{Command, Confirm};
use super::parse;

fn confirm(argv: &[&str]) -> Option<Confirm> {
    parse(argv).unwrap().command.unwrap().confirm()
}

#[test]
fn a_change_takes_json_next_to_yes_or_dry_run() {
    let Some(Command::Update(update)) = parse(&["update", "--json", "--yes"]).unwrap().command
    else {
        panic!("not update");
    };

    assert!(update.confirm.json);
    assert!(update.confirm.yes);
    assert!(confirm(&["update"]).is_some_and(|confirm| !confirm.json));
}

#[test]
fn json_without_an_answer_cannot_go_on() {
    let unanswered = Confirm {
        json: true,
        ..Confirm::default()
    };

    let yes = Confirm {
        yes: true,
        ..unanswered
    };

    let dry_run = Confirm {
        dry_run: true,
        ..unanswered
    };

    assert!(unanswered.cannot_ask());
    assert!(!yes.cannot_ask());
    assert!(!dry_run.cannot_ask());
    assert!(!Confirm::default().cannot_ask());
}

#[test]
fn every_change_hands_out_its_question() {
    for argv in [
        &["update", "--json", "--yes"][..],
        &["services", "restart", "sshd.service", "--json", "--yes"],
        &["kernels", "install", "linux-lts", "--json", "--yes"],
        &["users", "remove", "ada", "--json", "--yes"],
        &["boot", "param", "add", "quiet", "--json", "--yes"],
        &["configs", "edit", "pacman-config", "--json", "--yes"],
    ] {
        assert!(
            confirm(argv).is_some_and(|confirm| confirm.json),
            "{argv:?}"
        );
    }

    assert!(confirm(&["status", "--json"]).is_none());
    assert!(confirm(&["kernels", "list", "--json"]).is_none());
}
