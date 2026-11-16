use strum::IntoEnumIterator as _;
use toolbox_plan::{PackageName, PlannedAction};

use super::{install, staged};
use crate::SummaryCode;
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

/// A code no domain gives would never be worded. Two domains sharing one
/// are refused by the registry itself.
#[test]
fn every_change_code_belongs_to_one_domain() {
    let domains = super::all().unwrap();
    let missing: Vec<SummaryCode> = SummaryCode::iter()
        .filter(|code| !domains.covers(*code))
        .collect();

    assert_eq!(missing, []);
}
