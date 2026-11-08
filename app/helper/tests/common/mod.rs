use std::error::Error;
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::process::{Command, ExitStatus, Stdio};

use toolbox_plan::protocol::{Answer, Event, Mode, Outcome, Request, decode, encode};
use toolbox_plan::{RootAction, Step, StepId};

pub type TestResult = Result<(), Box<dyn Error>>;

pub struct Run {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// A reply typed once the running program prints a question.
pub struct Reply<'a> {
    /// Text of the question, such as pacman's `[Y/n]`.
    pub asked: &'a str,
    pub typed: &'a str,
}

/// Sends `stdin` and keeps the pipe open until the helper says `done`, as
/// the app does: a closed stdin means the app is gone, and the helper then
/// cancels the running program on purpose.
pub fn helper(args: &[&str], stdin: &str) -> Result<Run, Box<dyn Error>> {
    helper_answering(args, stdin, None)
}

/// [`helper`] that answers one question of the running program, as a
/// person in the Console does. Input sent before the program runs is
/// dropped by the helper, so it waits for the question.
pub fn helper_answering(
    args: &[&str],
    stdin: &str,
    mut reply: Option<Reply<'_>>,
) -> Result<Run, Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut input = child.stdin.take().ok_or("no stdin pipe")?;
    let output = child.stdout.take().ok_or("no stdout pipe")?;
    let mut errors = child.stderr.take().ok_or("no stderr pipe")?;
    input.write_all(stdin.as_bytes())?;

    let mut stdout = String::new();

    for line in BufReader::new(output).lines() {
        let line = line?;
        let event = decode::<Event>(&line).ok();
        let done = event == Some(Event::Done);

        if let Some(Event::Output { step: _, data }) = &event
            && let Some(answer) = reply.take_if(|answer| data.contains(answer.asked))
        {
            input.write_all(
                encode(&Answer::Input {
                    data: answer.typed.to_owned(),
                })?
                .as_bytes(),
            )?;
            input.flush()?;
        }

        stdout.push_str(&line);
        stdout.push('\n');

        if done {
            break;
        }
    }

    drop(input);

    let status = child.wait()?;

    // The helper writes little to stderr, so reading it last cannot block it.
    let mut stderr = String::new();

    errors.read_to_string(&mut stderr)?;

    Ok(Run {
        status,
        stdout,
        stderr,
    })
}

pub fn events(run: &Run) -> Result<Vec<Event>, Box<dyn Error>> {
    run.stdout
        .lines()
        .map(|line| decode::<Event>(line).map_err(Into::into))
        .collect()
}

pub fn request(mode: Mode, actions: Vec<RootAction>) -> Result<String, Box<dyn Error>> {
    let steps = (1_u32..)
        .zip(actions)
        .map(|(id, action)| Step {
            id: StepId::new(id),
            action,
        })
        .collect();

    Ok(encode(&Request::new(mode, steps))?)
}

pub fn outcomes(run: &Run) -> Result<Vec<Outcome>, Box<dyn Error>> {
    Ok(events(run)?
        .into_iter()
        .filter_map(|event| match event {
            Event::Finished { step: _, outcome } => Some(outcome),
            Event::Accepted
            | Event::Rejected { reason: _ }
            | Event::Started { step: _ }
            | Event::Output { step: _, data: _ }
            | Event::Done => None,
        })
        .collect())
}
