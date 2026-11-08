//! Output counts even when the program exits with an error, because tools
//! like garuda-health report findings through their exit code.

use std::fmt::Write as _;

use super::Collector;

const ESCAPE: char = '\u{1b}';

const RESET: &str = "\u{1b}[0m";

const CUT_NOTE: &str = "[The start of this output was left out, only its end is shown.]";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ran {
    /// The page shows the colour codes, the plain report leaves them out.
    pub output: String,
    pub errors: String,
    /// `None` when a signal ended the program.
    pub code: Option<i32>,
    pub truncated: bool,
}

/// The same four codes the app shows wherever a password dialog can fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Dismissed,
    Denied,
    NoAgent,
    /// The helper is not where pkexec looks for it.
    HelperMissing,
}

impl Refusal {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Dismissed => "dismissed",
            Self::Denied => "denied",
            Self::NoAgent => "no-agent",
            Self::HelperMissing => "helper-missing",
        }
    }

    const fn report_note(self) -> &'static str {
        match self {
            Self::Dismissed => "Not collected: the password dialog was dismissed.",
            Self::Denied => "Not collected: root rights were not given.",
            Self::NoAgent => "Not collected: no password dialog could appear.",
            Self::HelperMissing => "Not collected: the helper is not installed.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attempt {
    Ran(Ran),
    Missing,
    Refused(Refusal),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Report(String),
    Unavailable,
    Refused(Refusal),
    Failed(String),
}

impl Ran {
    pub fn succeeded(&self) -> bool {
        self.code == Some(0)
    }

    fn closed(&self) -> Self {
        Self {
            output: closed(&self.output),
            errors: closed(&self.errors),
            code: self.code,
            truncated: self.truncated,
        }
    }

    fn exit_note(&self) -> String {
        self.code.map_or_else(
            || "Ended by a signal.".to_owned(),
            |code| format!("Exit code: {code}"),
        )
    }
}

pub fn outcome(attempts: &[Attempt]) -> Outcome {
    let mut merged: Option<Ran> = None;

    for attempt in attempts {
        match (attempt, merged.as_mut()) {
            (Attempt::Ran(ran), None) => merged = Some(ran.closed()),
            (Attempt::Ran(ran), Some(sum)) => {
                end_line(&mut sum.output);
                sum.output.push('\n');
                sum.output.push_str(&closed(&ran.output));
                sum.errors.push_str(&closed(&ran.errors));
                sum.code = ran.code;
                sum.truncated |= ran.truncated;
            }
            (Attempt::Missing, None) => return Outcome::Unavailable,
            (Attempt::Refused(refusal), None) => return Outcome::Refused(*refusal),
            (Attempt::Failed(reason), None) => return Outcome::Failed(reason.clone()),
            // What ran before still counts. The rest shows as an error.
            (Attempt::Missing, Some(sum)) => fail(sum, "The next program is not installed."),
            (Attempt::Refused(refusal), Some(sum)) => fail(sum, refusal.report_note()),
            (Attempt::Failed(reason), Some(sum)) => fail(sum, reason),
        }
    }

    merged.map_or_else(
        || Outcome::Failed("No program ran.".to_owned()),
        |ran| render(&ran),
    )
}

pub fn full_report(parts: &[(Collector, Outcome)]) -> String {
    let mut report = String::new();

    for (collector, outcome) in parts {
        let name = collector.as_str();
        let mut body = match outcome {
            Outcome::Report(text) => text.clone(),
            Outcome::Unavailable => "Not available on this system.".to_owned(),
            Outcome::Refused(refusal) => refusal.report_note().to_owned(),
            Outcome::Failed(reason) => format!("Failed: {reason}"),
        };

        end_line(&mut body);

        // Writing to a string cannot fail.
        write!(report, "### {name} ###\n{body}### END {name} ###\n\n").ok();
    }

    report
}

/// The cut is at the last line matching `PACMAN.*pacman`, such as
/// `[PACMAN] Running 'pacman -Syu'`. Without one the whole text stays.
pub fn since_last_pacman_run(log: &str) -> (&str, bool) {
    let mut start = None;
    let mut offset = 0_usize;

    for line in log.split_inclusive('\n') {
        let is_run = line
            .split_once("PACMAN")
            .is_some_and(|(_, rest)| rest.contains("pacman"));

        if is_run {
            start = Some(offset);
        }

        offset = offset.saturating_add(line.len());
    }

    start
        .and_then(|start| log.get(start..))
        .map_or((log, false), |part| (part, true))
}

fn fail(ran: &mut Ran, reason: &str) {
    end_line(&mut ran.errors);
    ran.errors.push_str(reason);
    ran.code = ran.code.filter(|code| *code != 0_i32).or(Some(1_i32));
}

fn render(ran: &Ran) -> Outcome {
    let errors = ran.errors.trim();

    if ran.succeeded() {
        return Outcome::Report(with_cut_note(ran, ran.output.clone()));
    }

    if ran.output.trim().is_empty() {
        return Outcome::Failed(if errors.is_empty() {
            ran.exit_note()
        } else {
            errors.to_owned()
        });
    }

    let mut text = format!("{}\n\n", ran.output.trim_end());

    if !errors.is_empty() {
        text.push_str(errors);
        text.push('\n');
    }

    text.push_str(&ran.exit_note());
    text.push('\n');
    Outcome::Report(with_cut_note(ran, text))
}

fn with_cut_note(ran: &Ran, text: String) -> String {
    if ran.truncated {
        format!("{CUT_NOTE}\n{text}")
    } else {
        text
    }
}

fn end_line(text: &mut String) {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
}

/// holds colour codes. A program that ends without one would otherwise
/// colour the app's own notes and the next heading of a full report.
fn closed(text: &str) -> String {
    if !text.contains(ESCAPE) {
        return text.to_owned();
    }

    let body = text.trim_end();
    let blanks = text.get(body.len()..).unwrap_or_default();

    format!("{body}{RESET}{blanks}")
}
