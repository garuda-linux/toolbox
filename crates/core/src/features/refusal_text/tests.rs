use toolbox_plan::{InvalidFullName, InvalidHomeDirectory, Protection};

use super::RefusalText;
use crate::features::boot::edit::EditCode;
use crate::features::configs::session::ProblemCode;
use crate::features::users::input::InputProblem;
use crate::features::{ReasonCode, boot, kernels, locales, packages, users};

#[test]
fn the_code_is_the_kebab_name() {
    assert_eq!(kernels::Refusal::LastKernel.code(), "last-kernel");
}

#[test]
fn the_text_id_joins_the_prefix_and_the_code() {
    assert_eq!(
        kernels::Refusal::RunningUnknown.text_id(),
        "kernels-refusal-running-unknown"
    );
    assert_eq!(
        users::Refusal::HomeTaken.text_id(),
        "users-refused-home-taken"
    );
}

/// A package rule another page meets shows the Packages page's text.
#[test]
fn a_package_rule_keeps_its_text_and_reason() {
    let along = packages::queue::Refusal::TakesAlong(Protection::Boot);

    assert_eq!(
        kernels::Refusal::Package(along).text_id(),
        "protected-along"
    );
    assert_eq!(
        locales::Refusal::Package(along).text_id(),
        "protected-along"
    );
    assert_eq!(
        kernels::Refusal::Package(along).protection(),
        Some(Protection::Boot)
    );
    assert_eq!(kernels::Refusal::LastKernel.protection(), None);
}

#[test]
fn a_config_file_problem_has_a_text_of_the_configs_page() {
    assert_eq!(
        ProblemCode::ReadFailed.text_id(),
        "configs-problem-read-failed"
    );
    assert_eq!(ProblemCode::NoAgent.text_id(), "configs-problem-no-agent");
}

#[test]
fn a_boot_value_that_breaks_its_rule_names_the_rule() {
    assert_eq!(
        EditCode::Rule(ReasonCode::InvalidPath).text_id(),
        "boot-refused-invalid-path"
    );
    assert_eq!(
        EditCode::Refused(boot::Refusal::UnknownTheme).text_id(),
        "boot-refused-unknown-theme"
    );
    assert_eq!(EditCode::NeedsConfirm.code(), "needs-confirm");
}

#[test]
fn a_typed_account_value_takes_the_text_of_its_field() {
    let ids = [
        InputProblem::Name(ReasonCode::TooLong),
        InputProblem::Home(InvalidHomeDirectory::OutsideRoots),
        InputProblem::FullName(InvalidFullName::LeadingDash),
        InputProblem::Refused(users::Refusal::GroupExists),
        InputProblem::UidInvalid,
    ]
    .map(RefusalText::text_id);

    assert_eq!(
        ids,
        [
            "users-name-too-long",
            "users-home-outside-roots",
            "users-full-name-leading-dash",
            "users-refused-group-exists",
            "users-refused-uid-invalid",
        ]
    );
}
