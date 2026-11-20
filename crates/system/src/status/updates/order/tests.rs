use toolbox_plan::PackageName;
use toolbox_state::testing::aur::order::{
    MISSING_CODE, MISSING_STDOUT, OCTOPI_GIT_CODE, OCTOPI_GIT_STDOUT,
};

use super::AurOrderSource as _;
use crate::ProbeError;
use crate::error::InvalidValue;
use crate::status::updates::Paru;
use crate::test_support::fake_program;

fn octopi() -> PackageName {
    PackageName::new("octopi-git").unwrap()
}

fn code(text: &str) -> u8 {
    text.trim().parse().unwrap()
}

#[test]
fn the_adapter_reads_the_order_paru_prints() {
    let (_folder, fake) = fake_program(OCTOPI_GIT_STDOUT, "", code(OCTOPI_GIT_CODE));
    let order = Paru::with_program(fake).order(&octopi()).unwrap();

    assert_eq!(
        order.builds.last().map(|build| build.name.as_str()),
        Some("octopi-git")
    );
}

/// paru exits with 1 when it names a missing package, which is an answer.
#[test]
fn a_missing_package_is_an_answer_not_a_failure() {
    let asked = PackageName::new("no-such-package-toolbox").unwrap();
    let (_folder, fake) = fake_program(MISSING_STDOUT, "", code(MISSING_CODE));
    let order = Paru::with_program(fake).order(&asked).unwrap();

    assert_eq!(order.missing.len(), 1);
}

/// The answer for another package says nothing about the asked one.
#[test]
fn a_missing_line_about_another_package_alone_fails() {
    let (_folder, fake) = fake_program(MISSING_STDOUT, "", code(MISSING_CODE));

    assert!(matches!(
        Paru::with_program(fake).order(&octopi()),
        Err(error) if error.is_failure()
    ));
}

/// The octopi-git capture with a missing dependency added and paru's exit
/// code for a missing name: the asked package is still in the order.
#[test]
fn a_missing_dependency_beside_the_asked_build_is_an_answer() {
    let text = format!("{OCTOPI_GIT_STDOUT}MISSING no-such-package-toolbox\n");
    let (_folder, fake) = fake_program(&text, "", code(MISSING_CODE));
    let order = Paru::with_program(fake).order(&octopi()).unwrap();

    assert_eq!(order.missing.len(), 1);
}

/// The octopi-git capture without its target line, with paru's exit code
/// for a missing name.
#[test]
fn a_missing_line_without_the_asked_build_fails() {
    let text = format!(
        "{}MISSING no-such-package-toolbox\n",
        OCTOPI_GIT_STDOUT.replace("AUR TARGET octopi-git octopi-git\n", "")
    );

    let (_folder, fake) = fake_program(&text, "", code(MISSING_CODE));

    assert!(matches!(
        Paru::with_program(fake).order(&octopi()),
        Err(error) if error.is_failure()
    ));
}

#[test]
fn a_failing_paru_fails() {
    assert!(matches!(
        Paru::with_program("false").order(&octopi()),
        Err(error) if error.is_failure()
    ));
}

#[test]
fn a_missing_paru_is_unavailable() {
    assert_eq!(
        Paru::with_program("toolbox-system-no-such-program").order(&octopi()),
        Err(ProbeError::Unavailable)
    );
}

/// The capture with its target line cut short.
#[test]
fn a_broken_order_fails() {
    let cut =
        OCTOPI_GIT_STDOUT.replace("AUR TARGET octopi-git octopi-git", "AUR TARGET octopi-git");

    let (_folder, fake) = fake_program(&cut, "", code(OCTOPI_GIT_CODE));

    assert!(matches!(
        Paru::with_program(fake).order(&octopi()),
        Err(ProbeError::Invalid {
            what: _,
            error: InvalidValue::Order(_)
        })
    ));
}
