use tokio::sync::oneshot;
use toolbox_instance::bus::queue::{Ask, Desk as _, Question, Refusal};

use super::Arrivals;
use crate::command::CommandBus;
use crate::instance::WAITING_ASKS;

#[test]
fn a_question_past_the_bound_is_refused_at_once() {
    let arrivals = Arrivals::new(CommandBus::new().sender);
    let mut waiting = Vec::new();
    for _ in 0..WAITING_ASKS {
        let (answer, answered) = oneshot::channel();
        arrivals.ask(Ask {
            question: Question::Edit("{}".to_owned()),
            answer,
        });
        waiting.push(answered);
    }

    let (answer, mut refused) = oneshot::channel();
    arrivals.ask(Ask {
        question: Question::Changes,
        answer,
    });

    assert_eq!(refused.try_recv(), Ok(Err(Refusal::TooMany)));
    assert!(
        waiting
            .iter_mut()
            .all(|answered| answered.try_recv().is_err())
    );
    assert_eq!(arrivals.questions().len(), WAITING_ASKS);
}

#[test]
fn a_question_waits_for_the_app_in_its_order() {
    let arrivals = Arrivals::new(CommandBus::new().sender);
    let (first, _first_answer) = oneshot::channel();
    let (second, _second_answer) = oneshot::channel();

    arrivals.ask(Ask {
        question: Question::Changes,
        answer: first,
    });
    arrivals.ask(Ask {
        question: Question::Edit("{}".to_owned()),
        answer: second,
    });

    let questions: Vec<Question> = arrivals
        .questions()
        .into_iter()
        .map(|ask| ask.question)
        .collect();

    assert_eq!(
        questions,
        [Question::Changes, Question::Edit("{}".to_owned())]
    );
    assert!(arrivals.questions().is_empty());
}
