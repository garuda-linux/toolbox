use super::*;

#[test]
fn the_octopi_git_plan_installs_dependencies_then_builds_in_paru_order() {
    let order = octopi();
    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert_eq!(
        planned.actions,
        [
            review(&["alpm_octopi_utils-git", "octopi-git"]),
            install_dependencies(dependencies(&order)),
            build_aur("alpm_octopi_utils-git"),
            install_built(&["alpm_octopi_utils-git"], InstallReason::Dependency),
            build_aur("octopi-git"),
            install_built(&["octopi-git"], InstallReason::Explicit),
            remove_make_dependencies(make_names(&order)),
        ]
    );
}

#[test]
fn the_review_is_the_first_action_with_every_base_in_build_order() {
    let order = octopi();
    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert_eq!(
        planned.actions.first(),
        Some(&review(&["alpm_octopi_utils-git", "octopi-git"]))
    );
    assert_eq!(
        planned.bases,
        [package("alpm_octopi_utils-git"), package("octopi-git")]
    );
}

#[test]
fn the_make_only_repository_names_are_removed_last() {
    let order = octopi();
    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert_eq!(
        planned.actions.last(),
        Some(&remove_make_dependencies(make_names(&order)))
    );
    assert_eq!(make_names(&order).len(), 37);
}

#[test]
fn a_protected_make_only_name_is_left_out_and_the_others_stay() {
    // The MAKE line `docbook-xsl` is renamed to `polkit`, a protected package.
    let mut order = octopi();
    for line in &mut order.repo {
        if line.name.as_str() == "docbook-xsl" {
            line.name = package("polkit");
        }
    }

    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    let expected: Vec<PackageName> = make_names(&octopi())
        .into_iter()
        .filter(|name| name.as_str() != "docbook-xsl")
        .collect();

    assert_eq!(
        planned.actions.last(),
        Some(&remove_make_dependencies(expected))
    );
}

#[test]
fn an_aur_make_dependency_is_installed_as_a_dependency_and_listed_for_removal() {
    // The DEP build `alpm_octopi_utils-git` is changed to a MAKE build, as
    // paru prints one for a make dependency that is an AUR package.
    let mut order = octopi();
    for build in &mut order.builds {
        if build.name.as_str() == "alpm_octopi_utils-git" {
            build.role = Role::Make;
        }
    }

    let planned = plan_of(&["octopi-git"], std::slice::from_ref(&order));

    assert!(planned.actions.contains(&install_built(
        &["alpm_octopi_utils-git"],
        InstallReason::Dependency
    )));
    let mut expected = make_names(&order);
    expected.push(package("alpm_octopi_utils-git"));
    assert_eq!(
        planned.actions.last(),
        Some(&remove_make_dependencies(expected))
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
            review(&["octopi-git"]),
            install_dependencies(dependencies(&order)),
            build_aur("octopi-git"),
            install_built(&["alpm_octopi_utils-git"], InstallReason::Dependency),
            install_built(&["octopi-git"], InstallReason::Explicit),
            remove_make_dependencies(make_names(&order)),
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

#[test]
fn the_outline_of_octopi_names_its_bases_and_its_repository_dependencies() {
    let order = octopi();
    let outline = crate::features::packages::aur::build::outline(&order);

    assert_eq!(
        outline.bases,
        [package("alpm_octopi_utils-git"), package("octopi-git")]
    );
    let mut listed = outline.dependencies;
    listed.sort();
    let mut expected: Vec<PackageName> = dependencies(&order).iter().cloned().collect();
    expected.sort();

    assert_eq!(listed, expected);
}
