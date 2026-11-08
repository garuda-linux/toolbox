use tokio::sync::oneshot;
use toolbox_instance::bus::queue::{Ask, Desk as _, Question};

use super::Arrivals;
use crate::command::CommandBus;

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
