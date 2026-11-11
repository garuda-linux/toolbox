use std::collections::BTreeMap;

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

/// A code no domain gives would never be worded, one that two give says
/// the same sentence for two kinds of change.
#[test]
fn every_change_code_belongs_to_one_domain() {
    let mut owners: BTreeMap<SummaryCode, Vec<&str>> = BTreeMap::new();
    for (domain, code) in super::all().unwrap().codes() {
        owners.entry(code).or_default().push(domain);
    }

    let missing: Vec<SummaryCode> = SummaryCode::iter()
        .filter(|code| !owners.contains_key(code))
        .collect();

    let shared: Vec<(SummaryCode, Vec<&str>)> = owners
        .into_iter()
        .filter(|(_, domains)| domains.len() > 1)
        .collect();

    assert_eq!(missing, []);
    assert_eq!(shared, []);
}

/// The planner orders changes by stage alone, so two domains on one
/// stage would fall back to an order nobody chose.
#[test]
fn every_domain_runs_at_a_stage_of_its_own() {
    let mut owners: BTreeMap<u16, Vec<&str>> = BTreeMap::new();
    for (domain, stage) in super::all().unwrap().stages() {
        owners.entry(stage.rank()).or_default().push(domain);
    }

    let shared: Vec<(u16, Vec<&str>)> = owners
        .into_iter()
        .filter(|(_, domains)| domains.len() > 1)
        .collect();

    assert_eq!(shared, []);
}
