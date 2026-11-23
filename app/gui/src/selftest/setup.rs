use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let go_to = Step::combo("Go to step");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Setup Assistant"),
        Step::wait(
            "Nothing is installed until you apply your choices at the end. Apps that are already installed show as on, and turning one off removes it.",
        ),
        Step::Shot("setup"),
        go_to.clone(),
        Step::click("Summary"),
        Step::WaitContaining("You picked nothing"),
        Step::Shot("setup-nothing"),
        go_to.clone(),
        Step::click("1. Audio"),
        Step::WaitContaining("Step 1 of"),
        Step::button("Install Strawberry Player"),
        Step::WaitContaining("with Apply"),
        Step::Shot("setup-step"),
        go_to,
        Step::click("Summary"),
        Step::wait("Apps to install: 1"),
        Step::Shot("setup-summary"),
        Step::click("Review and Apply"),
        Step::WaitContaining("strawberry"),
        Step::click("Apply the change"),
        Step::wait("Test mode: nothing was changed"),
        // The dry run installs nothing, so the queue is emptied the way the
        // next read of a real system would. On the Summary the queue shows
        // only for its review, which the Console would cover.
        Step::click("Hide the Console"),
        Step::click("Review and Apply"),
        Step::click("Remove All…"),
        Step::click("Remove All"),
        Step::wait("Success!"),
        Step::Shot("setup-success"),
        Step::click("Finish Setup"),
        Step::WaitGone("Success!"),
    ]
}
