use toolbox_core::ChangeSet;
use toolbox_core::features::kernels::Refusal;

use super::Kernels;
use crate::command::{AppCommand, KernelCommand, KernelEdit};
use crate::pages::kernels::fixtures::{package, system};

fn toggle(kernel: &str, on: bool) -> AppCommand {
    AppCommand::Kernel(KernelCommand::Edit(KernelEdit::Toggle {
        kernel: package(kernel),
        on,
    }))
}

#[test]
fn removing_the_running_kernel_is_refused_and_queues_nothing() {
    let mut kernels = Kernels::default();

    let handled = kernels.handle(&toggle("linux", false), &ChangeSet::default(), &system());

    assert!(handled.edit.is_none());
    let refused = kernels.refusal().unwrap();
    assert_eq!(refused.refusal, Refusal::Running);
    assert_eq!(refused.subject, package("linux"));
}

#[test]
fn a_taken_click_answers_with_its_edit_and_clears_the_refusal() {
    let mut kernels = Kernels::default();
    kernels.handle(&toggle("linux", false), &ChangeSet::default(), &system());

    let handled = kernels.handle(&toggle("linux-zen", true), &ChangeSet::default(), &system());

    assert!(handled.edit.is_some_and(|edit| !edit.add.is_empty()));
    assert!(kernels.refusal().is_none());
}

#[test]
fn dismissing_takes_the_refusal_away() {
    let mut kernels = Kernels::default();
    kernels.handle(&toggle("linux", false), &ChangeSet::default(), &system());

    kernels.handle(
        &AppCommand::Kernel(KernelCommand::DismissRefusal),
        &ChangeSet::default(),
        &system(),
    );

    assert!(kernels.refusal().is_none());
}

#[test]
fn another_page_s_command_is_not_taken() {
    let mut kernels = Kernels::default();

    let handled = kernels.handle(&AppCommand::ReadSystem, &ChangeSet::default(), &system());

    assert!(handled.edit.is_none());
    assert_eq!(handled.answers, []);
}
