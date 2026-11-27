//! A helper running a script of protocol lines, and user steps that hold
//! for a while and note what they were told in a log file.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use tokio::process::Command as Process;
use tokio::sync::mpsc::unbounded_channel;
use toolbox_exec::EventSink;
use toolbox_plan::protocol::{Answer, Event, Mode, Outcome, encode};
use toolbox_plan::{Plan, Step, UserAction};

use crate::apply::{ApplyCommand, Launcher, OutputGate, PlanId, Report, Reporter, serve};

/// One step of a fake helper's script, run by `sh`.
pub enum Act {
    /// Writes these events, one line each.
    Say(Vec<Event>),
    /// Reads one line from stdin and appends it to `record`.
    Hear(PathBuf),
    /// Notes in `log` how many lines `record` holds, then.
    Seen {
        record: PathBuf,
        log: PathBuf,
    },
    Note {
        log: PathBuf,
        text: &'static str,
    },
    Pause,
    /// Appends the helper's pid to `file`, once per start.
    Mark(PathBuf),
    Raw(&'static str),
}

/// What a user step does: it holds for `hold` unless a cancel comes, then
/// ends with `outcome`. Its answers are noted in `log`.
pub struct UserFake {
    pub log: PathBuf,
    pub hold: Duration,
    pub outcome: Outcome,
}

impl UserFake {
    fn run(&self, answers: &Receiver<Answer>) -> Outcome {
        note(&self.log, "user-start");
        let until = Instant::now() + self.hold;
        let outcome = loop {
            if Instant::now() >= until {
                break self.outcome.clone();
            }

            match answers.recv_timeout(Duration::from_millis(10)) {
                Ok(Answer::Cancel) | Err(RecvTimeoutError::Disconnected) => {
                    break Outcome::Cancelled;
                }
                Ok(Answer::Input { data }) => note(&self.log, &format!("input {data}")),
                Ok(Answer::Resize { size }) => {
                    note(&self.log, &format!("resize {} {}", size.columns, size.rows));
                }
                Ok(Answer::Finish | Answer::Proceed { step: _ })
                | Err(RecvTimeoutError::Timeout) => {}
            }
        };

        note(&self.log, "user-end");
        outcome
    }
}

/// The helper and the user steps of one test.
pub struct Scripted {
    pub script: String,
    pub mode: Mode,
    pub user: UserFake,
}

impl Launcher for Scripted {
    fn command(&self) -> Process {
        let mut command = Process::new("sh");
        command.arg("-c").arg(&self.script);
        command
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn user_steps(
        &self,
        steps: &[Step<UserAction>],
        answers: Receiver<Answer>,
        sink: &mut dyn EventSink,
    ) -> bool {
        for step in steps {
            sink.send(Event::Started { step: step.id });
            let outcome = if self.mode == Mode::DryRun {
                Outcome::Skipped
            } else {
                self.user.run(&answers)
            };

            let ok = matches!(outcome, Outcome::Succeeded | Outcome::Skipped);
            sink.send(Event::Finished {
                step: step.id,
                outcome,
            });

            if !ok {
                return false;
            }
        }

        true
    }
}

/// Serves `plan` and sends `then` once the report `when` matches. The
/// sender is dropped then, so the service ends with the plan.
pub async fn serve_when(
    launcher: Scripted,
    plan: Plan,
    when: impl Fn(&Report) -> bool,
    then: Vec<ApplyCommand>,
) -> Vec<Report> {
    let (commands, command_queue) = unbounded_channel();
    let (reports, report_queue) = crate::waking::channel(crate::waking::Wake::none());
    let reporter = Reporter {
        reports,
        gate: OutputGate::default(),
    };

    commands
        .send(ApplyCommand::Run {
            id: PlanId::new(1),
            plan,
        })
        .unwrap();
    let serving = tokio::spawn(serve(command_queue, Arc::new(launcher), reporter));

    let mut pending = Some((commands, then));
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !serving.is_finished() {
        assert!(Instant::now() < deadline, "the plan did not end: {seen:?}");

        for report in report_queue.try_iter() {
            if when(&report)
                && let Some((sender, queued)) = pending.take()
            {
                for command in queued {
                    sender.send(command).unwrap();
                }
            }

            seen.push(report);
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    serving.await.unwrap();
    seen.extend(report_queue.try_iter());
    seen
}

pub fn note(path: &Path, text: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    writeln!(file, "{text}").unwrap();
}

/// The lines a file holds, none when it does not exist.
pub fn lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// How often the helper started, by the pids its script appended.
pub fn starts(file: &Path) -> usize {
    lines(file).len()
}

/// Whether the last started helper is gone from the process table,
/// which it is only once its parent has reaped it.
pub fn reaped(file: &Path) -> bool {
    let pid = lines(file).last().cloned().unwrap_or_default();

    !Path::new("/proc").join(pid).exists()
}

/// The line the app sends for `answer`, without its newline.
pub fn wire(answer: &Answer) -> String {
    encode(answer).unwrap().trim_end().to_owned()
}

pub fn helper(acts: &[Act]) -> String {
    acts.iter().map(act).collect::<Vec<_>>().join("; ")
}

fn act(act: &Act) -> String {
    match act {
        Act::Say(events) => {
            let lines: String = events.iter().map(|event| encode(event).unwrap()).collect();
            format!("printf '%s' '{lines}'")
        }
        Act::Hear(record) => format!(
            "IFS= read -r line; printf '%s\\n' \"$line\" >> '{}'",
            record.display()
        ),
        Act::Seen { record, log } => format!(
            "printf 'seen %s\\n' \"$(wc -l < '{}')\" >> '{}'",
            record.display(),
            log.display()
        ),
        Act::Note { log, text } => format!("printf '%s\\n' '{text}' >> '{}'", log.display()),
        Act::Pause => "sleep 0.3".to_owned(),
        Act::Mark(file) => format!("echo $$ >> '{}'", file.display()),
        Act::Raw(line) => (*line).to_owned(),
    }
}
