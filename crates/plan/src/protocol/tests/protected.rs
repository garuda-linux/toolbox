use super::{request, step};
use crate::protocol::RequestError;
use crate::{PackageName, RootAction, StepId, TargetList};

fn list(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(names.iter().map(|name| PackageName::new(*name).unwrap())).unwrap()
}

fn removal(names: &[&str]) -> RootAction {
    RootAction::RemovePackages {
        packages: list(names),
    }
}

#[test]
fn a_removal_of_a_protected_package_is_refused() {
    let asked = request(vec![step(1, removal(&["gimp", "sudo"]))]);

    assert_eq!(
        asked.validate().unwrap_err(),
        RequestError::Protected {
            step: StepId::new(1),
            package: PackageName::new("sudo").unwrap(),
        }
    );
}

#[test]
fn a_removal_of_unprotected_packages_passes() {
    let asked = request(vec![step(1, removal(&["gimp", "gegl"]))]);

    assert_eq!(asked.validate(), Ok(()));
}

fn make_dependency_removal(names: &[&str]) -> RootAction {
    RootAction::RemoveMakeDependencies {
        packages: list(names),
    }
}

#[test]
fn a_removal_of_a_protected_make_dependency_is_refused() {
    let asked = request(vec![step(1, make_dependency_removal(&["cowsay", "bash"]))]);

    assert_eq!(
        asked.validate().unwrap_err(),
        RequestError::Protected {
            step: StepId::new(1),
            package: PackageName::new("bash").unwrap(),
        }
    );
}

#[test]
fn a_removal_of_unprotected_make_dependencies_passes() {
    let asked = request(vec![step(1, make_dependency_removal(&["cowsay"]))]);

    assert_eq!(asked.validate(), Ok(()));
}
