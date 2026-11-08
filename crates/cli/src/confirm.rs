//! The question before a change. Without a terminal nobody can answer, so a
//! change needs `--yes` there. A change that destroys something wants its
//! name typed.

#[cfg(test)]
mod tests;

use crate::exit::Exit;

/// The terminal the question is asked on. A port, so the tests need none.
pub trait Prompt {
    fn is_terminal(&self) -> bool;

    /// The answer's line, `None` once the input is closed.
    fn ask(&mut self, question: &str) -> Option<String>;

    /// A secret typed after `question` without echo. `None` when it
    /// cannot be read so, never an echoed fallback.
    fn secret(&mut self, _question: &str) -> Option<String> {
        None
    }

    /// The first line of standard input, for `--password-stdin`.
    fn line(&mut self) -> Option<String> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consent {
    Given,
    Declined,
    NeedsYes,
}

impl Consent {
    /// The exit code when nothing is carried out.
    pub const fn exit(self) -> Option<Exit> {
        match self {
            Self::Given => None,
            Self::Declined => Some(Exit::Declined),
            Self::NeedsYes => Some(Exit::NeedsYes),
        }
    }
}

/// `typed` names a destructive change, which wants that name as the answer.
pub fn consent(prompt: &mut dyn Prompt, yes: bool, typed: Option<&str>) -> Consent {
    if yes {
        return Consent::Given;
    }

    if !prompt.is_terminal() {
        return Consent::NeedsYes;
    }

    let agreed = match typed {
        Some(name) => prompt
            .ask(&format!("This cannot be undone. Type {name} to go on: "))
            .is_some_and(|answer| answer.trim() == name),
        None => prompt.ask("Continue? [y/N] ").is_some_and(|answer| {
            let answer = answer.trim();
            answer.eq_ignore_ascii_case("y") || answer.eq_ignore_ascii_case("yes")
        }),
    };

    if agreed {
        Consent::Given
    } else {
        Consent::Declined
    }
}
