use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

use super::{ANSWER_LIMIT, Ask, Desk, MAX_QUEUE_TEXT, Question, Queue, Refusal, refusal};

/// Answers every question as told, keeps the questions, or keeps the
/// answer channel and never answers.
struct Clerk {
    answer: Option<Result<String, Refusal>>,
    asked: Mutex<Vec<Question>>,
    pending: Mutex<Vec<Ask>>,
}

impl Clerk {
    fn answering(answer: Option<Result<String, Refusal>>) -> Arc<Self> {
        Arc::new(Self {
            answer,
            asked: Mutex::new(Vec::new()),
            pending: Mutex::new(Vec::new()),
        })
    }
}

impl Desk for Clerk {
    fn ask(&self, ask: Ask) {
        match self.answer.clone() {
            Some(answer) => {
                self.asked.lock().push(ask.question);
                ask.answer.send(answer).ok();
            }
            None => self.pending.lock().push(ask),
        }
    }
}

fn queue(clerk: &Arc<Clerk>) -> Queue {
    Queue {
        desk: desk(clerk),
        limit: ANSWER_LIMIT,
    }
}

fn desk(clerk: &Arc<Clerk>) -> Arc<dyn Desk> {
    Arc::<Clerk>::clone(clerk)
}

fn code(error: zbus::fdo::Error) -> String {
    let zbus::fdo::Error::Failed(code) = error else {
        panic!("not a refusal: {error}");
    };

    code
}

#[tokio::test]
async fn the_window_answers_with_its_queue() {
    let clerk = Clerk::answering(Some(Ok("{\"version\":5}".to_owned())));

    assert_eq!(queue(&clerk).changes().await.unwrap(), "{\"version\":5}");
    assert_eq!(*clerk.asked.lock(), [Question::Changes]);
}

#[tokio::test]
async fn an_edit_reaches_the_window_as_sent() {
    let clerk = Clerk::answering(Some(Ok(String::new())));

    queue(&clerk).edit("{\"add\":[]}".to_owned()).await.unwrap();

    assert_eq!(
        *clerk.asked.lock(),
        [Question::Edit("{\"add\":[]}".to_owned())]
    );
}

#[tokio::test]
async fn the_window_s_refusal_crosses_as_its_code() {
    let clerk = Clerk::answering(Some(Err(Refusal::Busy)));

    let error = queue(&clerk).edit("{}".to_owned()).await.unwrap_err();

    assert_eq!(code(error), "busy");
}

#[tokio::test]
async fn a_text_too_large_never_reaches_the_window() {
    let clerk = Clerk::answering(Some(Ok(String::new())));

    let error = queue(&clerk)
        .edit("x".repeat(MAX_QUEUE_TEXT + 1))
        .await
        .unwrap_err();

    assert_eq!(code(error), "too-large");
    assert!(clerk.asked.lock().is_empty());
}

#[tokio::test(start_paused = true)]
async fn a_window_that_does_not_answer_is_no_answer() {
    let clerk = Clerk::answering(None);
    let silent = Queue {
        desk: desk(&clerk),
        limit: Duration::from_secs(1),
    };

    let error = silent.changes().await.unwrap_err();

    assert_eq!(code(error), "no-answer");
    assert_eq!(clerk.pending.lock().len(), 1);
}

#[tokio::test]
async fn a_closing_window_is_no_answer() {
    struct Gone;
    impl Desk for Gone {
        fn ask(&self, ask: Ask) {
            drop(ask);
        }
    }

    let gone = Queue {
        desk: Arc::new(Gone),
        limit: ANSWER_LIMIT,
    };

    assert_eq!(code(gone.changes().await.unwrap_err()), "no-answer");
}

#[test]
fn every_code_names_its_refusal_again() {
    for refusal in [
        Refusal::Busy,
        Refusal::Invalid,
        Refusal::TooLarge,
        Refusal::NoAnswer,
        Refusal::Unsupported,
    ] {
        assert_eq!(Refusal::from_code(refusal.code()), Some(refusal));
    }

    assert_eq!(Refusal::from_code("format-disk"), None);
}

#[test]
fn an_older_window_without_the_interface_is_unsupported() {
    let missing = zbus::Error::FDO(Box::new(zbus::fdo::Error::UnknownInterface(
        "org.garudalinux.toolbox.Queue".to_owned(),
    )));

    let busy = zbus::Error::FDO(Box::new(zbus::fdo::Error::Failed("busy".to_owned())));

    assert_eq!(refusal(&missing), Refusal::Unsupported);
    assert_eq!(refusal(&busy), Refusal::Busy);
    assert_eq!(refusal(&zbus::Error::InvalidReply), Refusal::NoAnswer);
}
