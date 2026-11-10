//! `garuda-toolbox doctor`: what the toolbox needs, each part with whether
//! it is there and works, and the fix of what is not. It only reads: no
//! root, no helper, no change.

mod checks;
mod facts;
#[cfg(test)]
mod tests;

use std::fmt::Write as _;
use std::io::Write as _;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::Read;
use toolbox_cli::output::{Mark, Paint, Row, json};

pub use self::checks::checks;
pub use self::facts::Facts;
use super::spinner;
use crate::out::Out;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Ok,
    /// Something the toolbox needs is missing or broken.
    Problem,
    /// Not installed, and only one feature misses it.
    Optional,
    /// Not found, but the toolbox cannot tell for sure.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part {
    pub id: &'static str,
    pub name: &'static str,
}

impl Part {
    pub const fn new(id: &'static str, name: &'static str) -> Self {
        Self { id, name }
    }

    pub const fn named(id: &'static str) -> Self {
        Self { id, name: id }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Check {
    /// Stable for scripts, while `name` may be worded anew.
    pub id: &'static str,
    pub name: &'static str,
    #[serde(rename = "result")]
    pub state: State,
    pub text: String,
    pub hint: Option<String>,
}

pub fn needs_fixing(checks: &[Check]) -> bool {
    checks.iter().any(|check| check.state == State::Problem)
}

pub fn text(checks: &[Check], paint: Paint) -> String {
    let rows: Vec<Row> = checks
        .iter()
        .map(|check| {
            Row::new(
                mark(check.state),
                vec![check.name.to_owned(), check.text.clone()],
            )
        })
        .collect();

    let mut text = paint.list("Doctor", Some(&summary(checks)), &[], &rows);
    let mut hints = checks
        .iter()
        .filter(|check| check.state == State::Problem)
        .filter_map(|check| Some((check.name, check.hint.as_deref()?)))
        .peekable();

    if hints.peek().is_some() {
        text.push('\n');
    }

    for (name, hint) in hints {
        writeln!(text, "{}", paint.hint(&format!("{name}: {hint}"))).ok();
    }

    text
}

/// Gathers the facts, prints the checks and exits 9 when one needs a fix.
pub fn run(read: Read, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let facts = spinner::while_reading(read.json || out.quiet(), || facts::gather(runtime));
    let found = checks(&facts);

    let shown = if read.json {
        json(&found)
    } else {
        text(&found, out.paint())
    };

    write!(out, "{shown}").ok();
    if needs_fixing(&found) {
        Exit::Attention
    } else {
        Exit::Success
    }
}

const fn mark(state: State) -> Mark {
    match state {
        State::Ok => Mark::On,
        State::Problem => Mark::Problem,
        State::Optional => Mark::Off,
        State::Unknown => Mark::Unknown,
    }
}

fn summary(checks: &[Check]) -> String {
    let count = |state: State| checks.iter().filter(|check| check.state == state).count();
    let problems = count(State::Problem);

    [
        Some(format!("{} ok", count(State::Ok))),
        (problems > 0).then(|| match problems {
            1 => "1 problem".to_owned(),
            many => format!("{many} problems"),
        }),
        Some(count(State::Optional))
            .filter(|optional| *optional > 0)
            .map(|optional| format!("{optional} not installed")),
        Some(count(State::Unknown))
            .filter(|unknown| *unknown > 0)
            .map(|unknown| format!("{unknown} unknown")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}
