use crate::features::configs::session::{Phase, Stage};

#[test]
fn only_a_starting_or_running_editor_is_editing() {
    let editing: Vec<Phase> = [
        Phase::Idle,
        Phase::Reading,
        Phase::Starting(Stage::Editor),
        Phase::Running(Stage::Editor),
        Phase::Starting(Stage::Write),
        Phase::Running(Stage::Write),
        Phase::Comparing,
        Phase::Review,
        Phase::Saved,
        Phase::Unchanged,
        Phase::Failed,
    ]
    .into_iter()
    .filter(|phase| phase.is_editing())
    .collect();

    assert_eq!(
        editing,
        [
            Phase::Starting(Stage::Editor),
            Phase::Running(Stage::Editor)
        ]
    );
}
