use toolbox_plan::procedure::Builtin;
use toolbox_plan::{PackageName, TargetList};

use super::support::{
    Fetch, OCTOPI, QUESTION, case, commit, enter_at, git, marker_program, review, shown,
};
use crate::test_support::{ada_home, assert_failed, run_builtin};

#[test]
fn a_repository_config_cannot_run_a_diff_driver() {
    let case = case(Fetch::Copy);
    let (program, marker) = marker_program(&case.sources, "external");
    git(
        &case.sources.join(OCTOPI),
        &["config", "diff.external", program.to_str().unwrap()],
    );

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(!marker.exists());
    assert!(shown(&session).contains("+pkgver=1"));
}

#[test]
fn a_textconv_driver_named_by_the_attributes_cannot_run() {
    let case = case(Fetch::Copy);
    let source = case.sources.join(OCTOPI);
    let (program, marker) = marker_program(&case.sources, "textconv");
    git(
        &source,
        &["config", "diff.evil.textconv", program.to_str().unwrap()],
    );
    commit(
        &source,
        &[
            (".gitattributes", "PKGBUILD diff=evil\n"),
            ("PKGBUILD", "pkgname=octopi-git\npkgver=2\n"),
        ],
    );

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert!(!marker.exists());
    assert!(shown(&session).contains("+pkgver=2"));
}

#[test]
fn the_helper_fails_a_review_because_it_has_no_console() {
    let (root, _home) = ada_home();
    let bases = TargetList::new([PackageName::new(OCTOPI).unwrap()]).unwrap();

    let (outcome, printed) = run_builtin(root.path(), &Builtin::ReviewAur { bases });

    assert_failed(outcome);
    assert!(printed.contains("needs a runner"), "{printed}");
}
