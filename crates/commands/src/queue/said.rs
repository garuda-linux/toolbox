//! What the queue commands print: each failure as an `error:` line with
//! its hint, and how an edit ended.

use std::io::Write;

use toolbox_cli::Exit;
use toolbox_core::Summary;
use toolbox_instance::bus::queue::Refusal;

use super::Failed;
use super::store::{Problem, Store};
use crate::out::{failure, say_error};
use crate::paint;

impl Failed {
    pub const fn exit(&self) -> Exit {
        match self {
            Self::Refused(refused) => refused.exit(),
            Self::Unpicked(unpicked) => unpicked.exit(),
            Self::Store(_) => Exit::Failure,
            Self::Nothing => Exit::Success,
        }
    }

    pub(super) fn say(&self, out: &mut dyn Write) -> Exit {
        match self {
            Self::Refused(refused) => {
                writeln!(out, "{}", refused.line(paint())).ok();
            }
            Self::Unpicked(unpicked) => match unpicked.message().split_once('\n') {
                Some((error, hint)) => say(out, error, hint),
                None => say_error(out, unpicked.message()),
            },
            Self::Store(Problem::File(raw)) => failure(out, "the queue could not be saved", raw),
            Self::Store(Problem::Window(refusal)) => {
                let (error, hint) = refused_words(*refusal);
                say(out, error, hint);
            }
            Self::Nothing => {
                writeln!(out, "Nothing to do: the queue is like this already.").ok();
            }
        }

        self.exit()
    }
}

const fn refused_words(refusal: Refusal) -> (&'static str, &'static str) {
    match refusal {
        Refusal::Busy => (
            "the window is applying its queue",
            "wait until it is done, then try again",
        ),
        Refusal::Invalid => (
            "the window did not take the change",
            "see the window's log for why",
        ),
        Refusal::TooLarge => (
            "the change is too large to hand to the window",
            "apply part of the queue in the window first",
        ),
        Refusal::TooMany => (
            "the window has too many queue commands waiting",
            "try again in a moment",
        ),
        Refusal::NoAnswer => (
            "the window did not answer",
            "try again, or close the window",
        ),
        Refusal::Unsupported => (
            "the running window takes no queue commands",
            "close the window and open it again",
        ),
    }
}

pub(super) fn say(out: &mut dyn Write, error: &str, hint: &str) {
    let paint = paint();
    writeln!(out, "{}\n{}", paint.error(error), paint.hint(hint)).ok();
}

/// What an edit ends with: where the queue is now, and what comes next.
pub(super) fn done(said: &str, store: &dyn Store, out: &mut dyn Write) -> Exit {
    let hint = if store.in_window() {
        "the window shows it under Changes"
    } else {
        "garuda-toolbox queue show, then garuda-toolbox queue apply"
    };

    writeln!(out, "{said}.\n{}", paint().hint(hint)).ok();
    Exit::Success
}

pub(super) fn waiting(count: usize) -> String {
    format!("{} waiting", changes(count))
}

pub(super) fn changes(count: usize) -> String {
    if count == 1 {
        "1 change".to_owned()
    } else {
        format!("{count} changes")
    }
}

/// A change in the words of the window's Changes list, in English as all
/// command line output, whatever the session.
pub(super) fn words() -> impl Fn(&Summary) -> String {
    let language = toolbox_services::i18n::FALLBACK;

    move |summary| toolbox_services::i18n::changes::sentence_in(&language, summary)
}
