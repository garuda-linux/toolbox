use toolbox_state::testing::aur::order::{
    AUR_SYNC_VOTE_STDOUT, FIREFOX_STDOUT, NVIDIA_SPLIT_STDOUT,
};

use super::*;

#[test]
fn a_queued_name_that_resolves_to_a_repository_is_installed_explicitly_without_a_build() {
    let order = capture(FIREFOX_STDOUT);
    let planned = plan_of(&["firefox"], std::slice::from_ref(&order));

    assert_eq!(planned.actions.len(), 2);
    assert_eq!(
        planned.actions[0],
        root(RootAction::InstallPackages {
            packages: targets(&["firefox"]),
        })
    );
    assert_eq!(
        planned.actions[1],
        install_dependencies(dependencies(&order))
    );
    assert_eq!(planned.bases, Vec::<PackageName>::new());
}

#[test]
fn a_build_with_make_lines_installs_its_dependencies_and_reports_the_make_names() {
    let order = capture(AUR_SYNC_VOTE_STDOUT);
    let planned = plan_of(&["aur-sync-vote"], std::slice::from_ref(&order));

    assert_eq!(
        planned.actions,
        [
            review(&["aur-sync-vote"]),
            install_dependencies(dependencies(&order)),
            build_aur("aur-sync-vote"),
            install_built(&["aur-sync-vote"], InstallReason::Explicit),
            remove_make_dependencies(make_names(&order)),
        ]
    );
    assert_eq!(make_names(&order).len(), 10);
}

#[test]
fn a_plan_of_repository_targets_has_no_review() {
    let order = capture(FIREFOX_STDOUT);
    let planned = plan_of(&["firefox"], std::slice::from_ref(&order));

    assert!(!planned.actions.iter().any(is_review));
    assert_eq!(planned.bases, Vec::<PackageName>::new());
}

#[test]
fn a_plan_without_make_only_names_has_no_removal() {
    let firefox = capture(FIREFOX_STDOUT);
    let nvidia = capture(NVIDIA_SPLIT_STDOUT);

    assert!(
        !plan_of(&["firefox"], std::slice::from_ref(&firefox))
            .actions
            .iter()
            .any(is_removal)
    );
    assert!(
        !plan_of(
            &["nvidia-470xx-utils", "opencl-nvidia-470xx"],
            std::slice::from_ref(&nvidia)
        )
        .actions
        .iter()
        .any(is_removal)
    );
}

#[test]
fn a_split_package_is_built_once_and_its_packages_are_installed() {
    let order = capture(NVIDIA_SPLIT_STDOUT);
    let planned = plan_of(
        &["nvidia-470xx-utils", "opencl-nvidia-470xx"],
        std::slice::from_ref(&order),
    );

    assert_eq!(
        planned.actions,
        [
            review(&["nvidia-470xx-utils"]),
            install_dependencies(dependencies(&order)),
            build_aur("nvidia-470xx-utils"),
            install_built(
                &["nvidia-470xx-utils", "opencl-nvidia-470xx"],
                InstallReason::Explicit
            ),
        ]
    );
    assert_eq!(planned.bases, [package("nvidia-470xx-utils")]);
}

#[test]
fn paru_prints_each_queued_name_as_a_target_in_its_own_capture() {
    let cases: [(&str, &[&str]); 4] = [
        (OCTOPI_GIT_STDOUT, &["octopi-git"]),
        (FIREFOX_STDOUT, &["firefox"]),
        (AUR_SYNC_VOTE_STDOUT, &["aur-sync-vote"]),
        (
            NVIDIA_SPLIT_STDOUT,
            &["nvidia-470xx-utils", "opencl-nvidia-470xx"],
        ),
    ];

    for (text, wanted) in cases {
        let order = capture(text);
        for name in wanted {
            assert!(is_target(&order, name), "{name} has no TARGET line");
        }
    }
}

fn is_target(order: &AurOrder, name: &str) -> bool {
    order
        .repo
        .iter()
        .any(|line| line.role == Role::Target && line.name.as_str() == name)
        || order
            .builds
            .iter()
            .any(|build| build.role == Role::Target && build.name.as_str() == name)
}
