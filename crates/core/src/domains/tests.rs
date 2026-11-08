use toolbox_plan::{PackageName, PlannedAction};

use super::{install, staged};
use crate::domain::{Stage, Staged};

#[test]
fn every_domain_registers_once() {
    super::all().unwrap();
}

fn step(stage: Stage, name: &str) -> Staged {
    let package = PackageName::new(name).unwrap();
    Staged::at(stage, PlannedAction::Root(install(&package)))
}

#[test]
fn staged_keeps_the_given_order_and_drops_empty_batches() {
    let first = step(Stage::TEARDOWN, "zsh");
    let second = step(Stage::PACKAGES, "bash");

    let plan = staged([Some(first.clone()), None, Some(second.clone())]);

    assert_eq!(plan, vec![first, second]);
}
