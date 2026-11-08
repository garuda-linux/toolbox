use std::io::Cursor;
use std::time::Duration;

use toolbox_plan::protocol::{Answer, MAX_LINE_BYTES, Mode, Request, RequestError, encode};
use toolbox_plan::{RootAction, Step, StepId};

use super::{read_request, spawn_answer_reader};

const SECRET: &str = "correct horse battery staple";

fn request_line() -> String {
    let request = Request::new(
        Mode::DryRun,
        vec![Step {
            id: StepId::new(1),
            action: RootAction::RefreshKeyring,
        }],
    );

    encode(&request).unwrap()
}

#[test]
fn the_first_line_is_the_request() {
    let mut input = Cursor::new(format!("{}{{\"type\":\"cancel\"}}\n", request_line()));

    let request = read_request(&mut input).unwrap();

    assert_eq!(request.mode, Mode::DryRun);
}

#[test]
fn an_empty_stdin_is_rejected() {
    let error = read_request(&mut Cursor::new("")).unwrap_err();

    assert_eq!(
        error,
        RequestError::Malformed("no request on stdin".to_owned())
    );
}

#[test]
fn an_overlong_request_is_rejected_without_parsing() {
    let line = format!("{}\n", "x".repeat(MAX_LINE_BYTES + 10));

    let error = read_request(&mut Cursor::new(line)).unwrap_err();

    assert_eq!(error, RequestError::Malformed("line too long".to_owned()));
}

#[test]
fn answers_after_the_request_arrive_in_order_and_garbage_is_skipped() {
    let mut input = Cursor::new(format!(
        "{}{}not json\n{}",
        request_line(),
        encode(&Answer::Input {
            data: "y\n".to_owned()
        })
        .unwrap(),
        encode(&Answer::Cancel).unwrap(),
    ));

    read_request(&mut input).unwrap();

    let answers = spawn_answer_reader(input);

    let timeout = Duration::from_secs(5);
    assert_eq!(
        answers.recv_timeout(timeout).unwrap(),
        Answer::Input {
            data: "y\n".to_owned()
        }
    );
    assert_eq!(answers.recv_timeout(timeout).unwrap(), Answer::Cancel);
    answers.recv_timeout(timeout).unwrap_err();
}

fn password_request(mode: Mode) -> Request {
    Request::new(
        mode,
        vec![
            Step {
                id: StepId::new(1),
                action: RootAction::RefreshKeyring,
            },
            Step {
                id: StepId::new(2),
                action: RootAction::SetPassword {
                    user: toolbox_plan::UserName::new("ada").unwrap(),
                    password: None,
                },
            },
        ],
    )
}

fn secret_of(request: &Request) -> Option<toolbox_plan::Password> {
    request
        .steps
        .iter()
        .find_map(|step| step.action.secret().cloned())
}

#[test]
fn the_secret_line_after_the_request_fills_its_step() {
    let mut request = password_request(Mode::Run);
    let mut input = Cursor::new(format!("secret 2 {SECRET}\n{{\"type\":\"cancel\"}}\n"));

    super::read_secrets(&mut input, &mut request).unwrap();

    assert_eq!(
        secret_of(&request),
        Some(toolbox_plan::Password::new(SECRET.to_owned()).unwrap())
    );

    // The answers behind the secret are still there for the runner.
    let answers = spawn_answer_reader(input);

    assert_eq!(
        answers.recv_timeout(Duration::from_secs(5)),
        Ok(Answer::Cancel)
    );
}

/// A secret longer than any secret line may be.
fn overlong() -> String {
    "x".repeat(toolbox_plan::MAX_PASSWORD_BYTES + super::SECRET_LINE_OVERHEAD + 1)
}

#[test]
fn a_missing_wrong_or_misplaced_secret_line_rejects_without_quoting_it() {
    for lines in [
        String::new(),
        format!("secret 1 {SECRET}\n"),
        format!("secret 2 {SECRET}"),
        format!("{{\"type\":\"input\",\"data\":\"{SECRET}\"}}\n"),
        format!("secret 2 {SECRET}\u{7}\n"),
        format!("secret 2 {}\n", overlong()),
    ] {
        let mut request = password_request(Mode::Run);
        let error = super::read_secrets(&mut Cursor::new(lines), &mut request).unwrap_err();

        assert_eq!(error, RequestError::MissingSecret(StepId::new(2)));
        assert_eq!(secret_of(&request), None);
    }
}

#[test]
fn an_overlong_secret_line_leaves_nothing_for_the_next_read() {
    let overlong = format!("secret 2 {}\n", overlong());
    let next = format!("secret 2 {SECRET}\n");
    let mut input = Cursor::new(format!("{overlong}{next}"));

    assert!(super::read_secret_line(&mut input).is_none());

    let line = super::read_secret_line(&mut input).unwrap();
    assert_eq!(line, next.as_bytes());
}

#[test]
fn a_dry_run_reads_no_secret_line() {
    let mut request = password_request(Mode::DryRun);
    let mut input = Cursor::new("{\"type\":\"cancel\"}\n".to_owned());

    super::read_secrets(&mut input, &mut request).unwrap();

    assert_eq!(secret_of(&request), None);
    assert_eq!(input.position(), 0);
}
