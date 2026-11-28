use toolbox_plan::procedure::Builtin;
use toolbox_plan::{PackageName, TargetList};

use super::support::{
    Fetch, OCTOPI, QUESTION, asked, case, commit, enter_at, git, marker_program, review, shown,
};
use crate::test_support::{ada_home, assert_failed, run_builtin};

#[test]
fn a_repository_config_cannot_run_a_diff_driver() {
    let case = case(Fetch::Tampered);
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
    let case = case(Fetch::Tampered);
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

/// With this attribute a plain checkout writes other bytes than git stores
/// and shows: the stored text reads as CJK characters, the file on disk as a
/// shell script that makepkg would run.
#[test]
fn a_file_is_on_disk_what_the_review_shows_whatever_its_attributes_say() {
    let case = case(Fetch::Copy);
    commit(
        &case.sources.join(OCTOPI),
        &[
            (
                ".gitattributes",
                "helper.sh working-tree-encoding=UTF-16LE\n",
            ),
            ("helper.sh", "echo ok\n"),
        ],
    );

    let (_, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    let on_disk = std::fs::read(case.clone_of(OCTOPI).join("helper.sh")).unwrap();
    let held = git(
        &case.clone_of(OCTOPI),
        &["cat-file", "blob", "HEAD:helper.sh"],
    );

    assert!(asked(&session));
    assert_ne!(on_disk, b"echo ok\n");
    assert_eq!(String::from_utf8_lossy(&on_disk).trim(), held);
}

/// A link in the build files would be reviewed as a path and built as what
/// it points at.
#[test]
fn a_link_among_the_build_files_is_refused() {
    let case = case(Fetch::Copy);
    let source = case.sources.join(OCTOPI);
    std::os::unix::fs::symlink("/etc/hostname", source.join("notes")).unwrap();
    commit(&source, &[]);

    let (outcome, session) = review(&case, &[OCTOPI], vec![enter_at(QUESTION)], None);

    assert_failed(outcome);
    assert!(!asked(&session));
}

#[test]
fn the_helper_fails_a_review_because_it_has_no_console() {
    let (root, _home) = ada_home();
    let bases = TargetList::new([PackageName::new(OCTOPI).unwrap()]).unwrap();

    let (outcome, printed) = run_builtin(root.path(), &Builtin::ReviewAur { bases });

    assert_failed(outcome);
    assert!(printed.contains("needs a runner"), "{printed}");
}
