mod cancel;
mod json;

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::process::Command as Process;
use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_cli::confirm::Prompt;
use toolbox_plan::protocol::{Event, Failure, Mode, Outcome, encode};
use toolbox_plan::{Plan, RootAction, StepId};
use toolbox_services::apply::Launcher;

use super::carry_out;

struct Script(String);

impl Launcher for Script {
    fn command(&self) -> Process {
        let mut command = Process::new("sh");
        command.arg("-c").arg(&self.0);
        command
    }

    fn mode(&self) -> Mode {
        Mode::DryRun
    }
}

struct Answers {
    terminal: bool,
    answers: VecDeque<&'static str>,
}

impl Prompt for Answers {
    fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn ask(&mut self, _question: &str) -> Option<String> {
        self.answers.pop_front().map(str::to_owned)
    }
}

struct Refusing {
    words: &'static str,
    then: Arc<Script>,
}

impl Launcher for Refusing {
    fn command(&self) -> Process {
        let mut command = Process::new("sh");
        command
            .arg("-c")
            .arg(format!("echo '{}' >&2; exit 127", self.words));
        command
    }

    fn mode(&self) -> Mode {
        Mode::Run
    }

    fn fallback(&self) -> Option<Arc<dyn Launcher>> {
        let script = Arc::clone(&self.then);
        let then: Arc<dyn Launcher> = script;
        Some(then)
    }
}

fn helper(outcome: Outcome) -> Arc<Script> {
    let step = StepId::new(0);
    let events: String = [
        Event::Accepted,
        Event::Started { step },
        Event::Output {
            step,
            data: "keyring refreshed\n".to_owned(),
        },
        Event::Finished { step, outcome },
        Event::Done,
    ]
    .iter()
    .map(|event| encode(event).unwrap())
    .collect();

    Arc::new(Script(format!("read request; printf '%s' '{events}'")))
}

fn plan() -> Plan {
    Plan::root(RootAction::RefreshKeyring)
}

fn run(plan: Plan, yes: bool, prompt: &mut Answers, outcome: Outcome) -> (Exit, String) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut out = Vec::new();
    let exit = carry_out(
        plan,
        Confirm {
            yes,
            dry_run: false,
            json: false,
        },
        None,
        runtime.handle(),
        helper(outcome),
        prompt,
        &mut out,
    );

    (exit, String::from_utf8(out).unwrap())
}

fn terminal(answers: &[&'static str]) -> Answers {
    Answers {
        terminal: true,
        answers: answers.iter().copied().collect(),
    }
}

#[test]
fn the_plan_is_shown_then_carried_out_with_its_output() {
    let (exit, out) = run(plan(), true, &mut terminal(&[]), Outcome::Succeeded);

    assert_eq!(exit, Exit::Success);
    assert!(
        out.starts_with("Planned  (1 step)\n\n  pacman-key"),
        "{out}"
    );
    assert!(out.contains("\nRunning\n"), "{out}");
    assert!(out.contains("keyring refreshed"), "{out}");
}

#[test]
fn a_no_changes_nothing() {
    let (exit, out) = run(plan(), false, &mut terminal(&["n"]), Outcome::Succeeded);

    assert_eq!(exit, Exit::Declined);
    assert!(!out.contains("keyring refreshed"), "{out}");
}

#[test]
fn without_a_terminal_it_wants_yes() {
    let mut piped = Answers {
        terminal: false,
        answers: VecDeque::new(),
    };

    let (exit, _) = run(plan(), false, &mut piped, Outcome::Succeeded);

    assert_eq!(exit, Exit::NeedsYes);
}

#[test]
fn a_failed_step_fails_the_command() {
    let (exit, _) = run(
        plan(),
        true,
        &mut terminal(&[]),
        Outcome::Failed {
            cause: Failure::ExitCode { code: 1_i32 },
            cleanup_failed: Vec::new(),
        },
    );

    assert_eq!(exit, Exit::Failure);
}

#[test]
fn an_empty_plan_has_nothing_to_ask() {
    let (exit, out) = run(Plan::new([]), false, &mut terminal(&[]), Outcome::Succeeded);

    assert_eq!(exit, Exit::Success);
    assert_eq!(out, "Nothing to do\n");
}

fn through(words: &'static str, prompt: &mut Answers) -> (Exit, String) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut out = Vec::new();
    let launcher = Arc::new(Refusing {
        words,
        then: helper(Outcome::Succeeded),
    });

    let exit = carry_out(
        plan(),
        Confirm {
            yes: true,
            dry_run: false,
            json: false,
        },
        None,
        runtime.handle(),
        launcher,
        prompt,
        &mut out,
    );

    (exit, String::from_utf8(out).unwrap())
}

#[test]
fn without_a_polkit_agent_a_terminal_asks_through_sudo() {
    let (exit, out) = through(
        "Error executing command as another user: No authentication agent found.",
        &mut terminal(&[]),
    );

    assert_eq!(exit, Exit::Success, "{out}");
    assert!(
        out.contains("\nwarning: pkexec is not available, asking through sudo\n"),
        "{out}"
    );
    assert!(out.contains("keyring refreshed"), "{out}");
    assert!(!out.contains("not carried out"), "{out}");
}

#[test]
fn a_pkexec_that_cannot_run_falls_back_to_sudo() {
    let (exit, out) = through("pkexec must be setuid root", &mut terminal(&[]));

    assert_eq!(exit, Exit::Success, "{out}");
    assert!(out.contains("asking through sudo"), "{out}");
}

#[test]
fn a_denied_password_is_never_asked_again_through_sudo() {
    let (exit, out) = through(
        "Error executing command as another user: Not authorized",
        &mut terminal(&[]),
    );

    assert_eq!(exit, Exit::Refused, "{out}");
    assert!(!out.contains("sudo"), "{out}");
}

#[test]
fn without_a_terminal_sudo_is_never_asked() {
    let mut piped = Answers {
        terminal: false,
        answers: VecDeque::new(),
    };

    let (exit, out) = through(
        "Error executing command as another user: No authentication agent found.",
        &mut piped,
    );

    assert_ne!(exit, Exit::Success, "{out}");
    assert!(!out.contains("sudo"), "{out}");
}

#[test]
fn a_dry_run_prints_the_plan_and_never_asks_or_runs() {
    let mut piped = Answers {
        terminal: false,
        answers: VecDeque::new(),
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut out = Vec::new();
    let confirm = Confirm {
        yes: false,
        dry_run: true,
        json: false,
    };

    let exit = carry_out(
        plan(),
        confirm,
        Some("alice"),
        runtime.handle(),
        helper(Outcome::Succeeded),
        &mut piped,
        &mut out,
    );

    let out = String::from_utf8(out).unwrap();

    assert_eq!(exit, Exit::Success, "{out}");
    assert!(
        out.starts_with("Planned  (1 step)\n\n  pacman-key"),
        "{out}"
    );
    assert!(
        out.ends_with("\nhint: run it without --dry-run to carry it out\n"),
        "{out}"
    );
    assert!(!out.contains("Running"), "{out}");
}
