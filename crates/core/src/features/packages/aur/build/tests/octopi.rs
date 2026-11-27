use super::*;

#[test]
fn the_octopi_git_plan_installs_dependencies_then_builds_in_paru_order() {
    let order = octopi();
    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert_eq!(
        planned.actions,
        [
            install_dependencies(dependencies(&order)),
            build_aur("alpm_octopi_utils-git"),
            install_built(&["alpm_octopi_utils-git"], InstallReason::Dependency),
            build_aur("octopi-git"),
            install_built(&["octopi-git"], InstallReason::Explicit),
        ]
    );
}

#[test]
fn the_make_only_repository_names_are_reported_for_removal() {
    let order = octopi();
    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert_eq!(planned.make_only.len(), 37);
    assert_eq!(planned.make_only, make_names(&order));
    assert_eq!(
        planned.bases,
        [package("alpm_octopi_utils-git"), package("octopi-git")]
    );
}

#[test]
fn packages_of_one_pkgbase_with_two_reasons_are_built_once_and_installed_per_reason() {
    // The DEP build takes the TARGET's pkgbase, as the two names of a split
    // package share one.
    let mut order = octopi();
    order.builds[0].base = package("octopi-git");

    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert_eq!(
        planned.actions,
        [
            install_dependencies(dependencies(&order)),
            build_aur("octopi-git"),
            install_built(&["alpm_octopi_utils-git"], InstallReason::Dependency),
            install_built(&["octopi-git"], InstallReason::Explicit),
        ]
    );
    assert_eq!(planned.bases, [package("octopi-git")]);
}

#[test]
fn an_installed_target_keeps_its_install_reason() {
    let order = octopi();
    let planned = plan(
        &queued(&["octopi-git"]),
        std::slice::from_ref(&order),
        &run(),
        &installed(&["octopi-git"]),
    )
    .unwrap();

    assert!(
        planned
            .actions
            .contains(&install_built(&["octopi-git"], InstallReason::Keep))
    );
    assert!(planned.actions.contains(&install_built(
        &["alpm_octopi_utils-git"],
        InstallReason::Dependency
    )));
}
