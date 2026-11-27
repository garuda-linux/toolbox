use toolbox_state::aur::order::AurBuild;

use super::*;

#[test]
fn a_queued_repository_name_is_installed_by_name_and_builds_nothing() {
    let order = repo_as("qt-sudo", Role::Target);
    let planned = plan_of(&["qt-sudo"], std::slice::from_ref(&order));

    let wanted = [
        root(RootAction::InstallPackages {
            packages: targets(&["qt-sudo"]),
        }),
        install_dependencies(dependencies(&order)),
        remove_make_dependencies(make_names(&order)),
    ];

    assert_eq!(planned.actions, wanted);
    assert_eq!(planned.bases, Vec::<PackageName>::new());
}

/// Stands in for paru's order of `alpm_octopi_utils-git` itself, where it is
/// the TARGET, with the repository lines of the octopi order.
fn alpm_target() -> AurOrder {
    let mut alpm = octopi();
    alpm.builds = vec![AurBuild {
        role: Role::Target,
        base: package("alpm_octopi_utils-git"),
        name: package("alpm_octopi_utils-git"),
    }];
    alpm
}

#[test]
fn two_queued_packages_sharing_a_dependency_build_and_install_it_once() {
    let orders = [octopi(), alpm_target()];
    let planned = plan_of(&["octopi-git", "alpm_octopi_utils-git"], &orders);

    assert_eq!(
        planned.actions,
        [
            review(&["alpm_octopi_utils-git", "octopi-git"]),
            install_dependencies(dependencies(&octopi())),
            build_aur("alpm_octopi_utils-git"),
            install_built(&["alpm_octopi_utils-git"], InstallReason::Explicit),
            build_aur("octopi-git"),
            install_built(&["octopi-git"], InstallReason::Explicit),
            remove_make_dependencies(make_names(&octopi())),
        ]
    );
    assert_eq!(
        planned.bases,
        [package("alpm_octopi_utils-git"), package("octopi-git")]
    );
}

#[test]
fn a_make_dependency_that_is_a_target_elsewhere_is_explicit_and_not_removed() {
    let orders = [octopi(), repo_as("libxt", Role::Target)];
    let planned = plan_of(&["octopi-git", "libxt"], &orders);

    assert_eq!(
        planned.actions.get(1),
        Some(&root(RootAction::InstallPackages {
            packages: targets(&["libxt"]),
        }))
    );
    let mut make = make_names(&octopi());
    make.retain(|name| name.as_str() != "libxt");
    assert_eq!(
        planned.actions.last(),
        Some(&remove_make_dependencies(make))
    );
}

#[test]
fn every_name_an_action_holds_was_named_by_an_order() {
    let mut split = octopi();
    split.builds[0].base = package("octopi-git");

    let cases = [
        (vec!["octopi-git"], vec![octopi()]),
        (vec!["octopi-git"], vec![split]),
        (vec!["qt-sudo"], vec![repo_as("qt-sudo", Role::Target)]),
        (
            vec!["octopi-git", "alpm_octopi_utils-git"],
            vec![octopi(), alpm_target()],
        ),
        (
            vec!["octopi-git", "libxt"],
            vec![octopi(), repo_as("libxt", Role::Target)],
        ),
    ];

    for (wished, orders) in cases {
        let named = named_by(&orders);
        let planned = plan_of(&wished, &orders);

        for action in &planned.actions {
            for name in names_of(action) {
                assert!(named.contains(&name), "{name} is named by no order");
            }
        }

        for base in &planned.bases {
            assert!(named.contains(base), "{base} is named by no order");
        }
    }
}
