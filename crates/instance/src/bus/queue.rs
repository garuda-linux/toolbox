//! The texts are JSON the app decodes and checks. This side only bounds
//! them, so the crate needs no queue types and a caller cannot park
//! megabytes in the app.

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;

use super::{BUS_NAME, OBJECT_PATH};

const INTERFACE: &str = "org.garudalinux.toolbox.Queue";

/// A whole queue fits many times over. Larger texts are refused unread.
pub const MAX_QUEUE_TEXT: usize = 256 * 1024;

/// How long a call waits for the window's answer. The window answers on
/// its next logic pass, which also runs while it is hidden.
pub const ANSWER_LIMIT: Duration = Duration::from_secs(5);

#[derive(Debug, PartialEq, Eq)]
pub enum Question {
    Changes,
    /// A queue edit as JSON, queued as the window's pages would.
    Edit(String),
}

/// Why the window did not do what a command asked. Crosses the bus as
/// its code, so the command picks its own words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Busy,
    Invalid,
    TooLarge,
    /// The window's queue of unanswered questions is full, so a flood of
    /// calls cannot pile up in it.
    TooMany,
    /// No answer in time, or the window is closing.
    NoAnswer,
    /// A window of an older version, without the interface.
    Unsupported,
}

#[derive(Debug)]
pub struct Ask {
    pub question: Question,
    pub answer: oneshot::Sender<Result<String, Refusal>>,
}

/// The app's side: takes a question on the bus's threads and answers it
/// later from its own.
pub trait Desk: Send + Sync + 'static {
    fn ask(&self, ask: Ask);
}

pub(super) struct Queue {
    pub(super) desk: Arc<dyn Desk>,
    pub(super) limit: Duration,
}

pub struct Window {
    connection: zbus::Connection,
}

impl Refusal {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Busy => "busy",
            Self::Invalid => "invalid",
            Self::TooLarge => "too-large",
            Self::TooMany => "too-many",
            Self::NoAnswer => "no-answer",
            Self::Unsupported => "unsupported",
        }
    }

    /// The refusal behind a code, `None` for one this version lacks.
    pub fn from_code(code: &str) -> Option<Self> {
        [
            Self::Busy,
            Self::Invalid,
            Self::TooLarge,
            Self::TooMany,
            Self::NoAnswer,
            Self::Unsupported,
        ]
        .into_iter()
        .find(|refusal| refusal.code() == code)
    }
}

#[zbus::interface(name = "org.garudalinux.toolbox.Queue")]
impl Queue {
    async fn changes(&self) -> zbus::fdo::Result<String> {
        self.pass(Question::Changes).await
    }

    /// Queues the edit in `change`, JSON the app checks before it touches
    /// its queue.
    async fn edit(&self, change: String) -> zbus::fdo::Result<()> {
        if change.len() > MAX_QUEUE_TEXT {
            return Err(failed(Refusal::TooLarge));
        }

        self.pass(Question::Edit(change)).await.map(drop)
    }
}

impl Queue {
    async fn pass(&self, question: Question) -> zbus::fdo::Result<String> {
        let (answer, answered) = oneshot::channel();
        self.desk.ask(Ask { question, answer });

        match tokio::time::timeout(self.limit, answered).await {
            Ok(Ok(Ok(text))) if text.len() <= MAX_QUEUE_TEXT => Ok(text),
            Ok(Ok(Ok(_too_long))) => Err(failed(Refusal::TooLarge)),
            Ok(Ok(Err(refusal))) => Err(failed(refusal)),
            Ok(Err(_)) | Err(_) => Err(failed(Refusal::NoAnswer)),
        }
    }
}

impl Window {
    /// The window that holds the session's name, `None` when no window
    /// runs or there is no session bus to reach one on.
    ///
    /// # Errors
    ///
    /// When the bus cannot say whether the name has an owner.
    pub async fn find() -> zbus::Result<Option<Self>> {
        let Ok(connection) = zbus::Connection::session().await else {
            return Ok(None);
        };

        let reply = connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "NameHasOwner",
                &(BUS_NAME,),
            )
            .await?;

        let runs: bool = reply.body().deserialize()?;

        Ok(runs.then_some(Self { connection }))
    }

    /// # Errors
    ///
    /// The window's refusal, or [`Refusal::NoAnswer`] for a bus error.
    pub async fn changes(&self) -> Result<String, Refusal> {
        let reply = answer(self.connection.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "Changes",
            &(),
        ))
        .await?;

        let text: String = reply.body().deserialize().map_err(|error| {
            tracing::warn!("the window's queue came in an unknown shape: {error}");
            Refusal::Invalid
        })?;

        if text.len() > MAX_QUEUE_TEXT {
            return Err(Refusal::TooLarge);
        }

        Ok(text)
    }

    /// # Errors
    ///
    /// The window's refusal, or [`Refusal::NoAnswer`] for a bus error.
    pub async fn edit(&self, change: &str) -> Result<(), Refusal> {
        if change.len() > MAX_QUEUE_TEXT {
            return Err(Refusal::TooLarge);
        }

        answer(self.connection.call_method(
            Some(BUS_NAME),
            OBJECT_PATH,
            Some(INTERFACE),
            "Edit",
            &(change,),
        ))
        .await
        .map(drop)
    }
}

async fn answer(
    call: impl Future<Output = zbus::Result<zbus::message::Message>>,
) -> Result<zbus::message::Message, Refusal> {
    match tokio::time::timeout(ANSWER_LIMIT + super::BUS_LIMIT, call).await {
        Ok(Ok(reply)) => Ok(reply),
        Ok(Err(error)) => Err(refusal(&error)),
        Err(_elapsed) => Err(Refusal::NoAnswer),
    }
}

fn failed(refusal: Refusal) -> zbus::fdo::Error {
    zbus::fdo::Error::Failed(refusal.code().to_owned())
}

fn refusal(error: &zbus::Error) -> Refusal {
    if let zbus::Error::FDO(fdo) = error {
        if let zbus::fdo::Error::Failed(code) = &**fdo {
            return Refusal::from_code(code).unwrap_or(Refusal::Invalid);
        }

        if matches!(
            &**fdo,
            zbus::fdo::Error::UnknownMethod(_)
                | zbus::fdo::Error::UnknownInterface(_)
                | zbus::fdo::Error::UnknownObject(_)
        ) {
            return Refusal::Unsupported;
        }
    }

    tracing::warn!("the window's queue did not answer: {error}");
    Refusal::NoAnswer
}
