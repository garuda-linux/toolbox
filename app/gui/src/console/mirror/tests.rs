use toolbox_plan::{PackageName, Plan, PlannedAction, RootAction, StepId, TargetList, Upgrader};

use super::MirrorWatch;

const MISSING: &str = "error: failed retrieving file 'gimp-3.0-1-x86_64.pkg.tar.zst' from mirror.example.org : The requested URL returned error: 404\r\n";

fn packages(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(
        names
            .iter()
            .map(|name| PackageName::new(*name).unwrap_or_else(|error| panic!("{error}"))),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

/// A system update, then an install of gimp: steps 1 and 2.
fn plan() -> Plan {
    Plan::new([
        PlannedAction::Root(RootAction::UpgradeSystem {
            upgrader: Upgrader::Pacman,
        }),
        PlannedAction::Root(RootAction::InstallPackages {
            packages: packages(&["gimp"]),
        }),
    ])
}

fn started(watch: &mut MirrorWatch) {
    watch.sent(&plan());
    watch.started();
}

#[test]
fn a_failed_install_that_met_a_missing_file_offers_the_retry() {
    let mut watch = MirrorWatch::default();
    started(&mut watch);

    watch.output(StepId::new(2), MISSING);
    watch.finished(StepId::new(2), false);

    assert_eq!(watch.offer(), Some(&packages(&["gimp"])));
}

#[test]
fn output_split_over_two_events_is_joined() {
    let mut watch = MirrorWatch::default();
    started(&mut watch);

    let (first, second) = MISSING.split_at(40);
    watch.output(StepId::new(2), first);
    watch.output(StepId::new(2), second);
    watch.finished(StepId::new(2), false);

    assert!(watch.offer().is_some());
}

#[test]
fn an_install_that_worked_or_failed_otherwise_offers_nothing() {
    let mut watch = MirrorWatch::default();
    started(&mut watch);
    watch.output(StepId::new(2), MISSING);
    watch.finished(StepId::new(2), true);
    assert_eq!(watch.offer(), None);

    started(&mut watch);
    watch.output(StepId::new(2), "error: target not found: gimp\n");
    watch.finished(StepId::new(2), false);
    assert_eq!(watch.offer(), None);
}

#[test]
fn a_missing_file_in_another_kind_of_step_offers_nothing() {
    let mut watch = MirrorWatch::default();
    started(&mut watch);

    watch.output(StepId::new(1), MISSING);
    watch.finished(StepId::new(1), false);

    assert_eq!(watch.offer(), None);
}

#[test]
fn the_next_plan_and_a_refused_one_keep_the_steps_apart() {
    let mut watch = MirrorWatch::default();
    started(&mut watch);
    watch.output(StepId::new(2), MISSING);
    watch.finished(StepId::new(2), false);

    // A plan the busy service refused never started.
    watch.sent(&Plan::root(RootAction::RefreshKeyring));
    watch.refused();
    assert!(watch.offer().is_some());

    watch.sent(&Plan::root(RootAction::RefreshKeyring));
    watch.started();
    assert_eq!(watch.offer(), None);

    watch.output(StepId::new(1), MISSING);
    watch.finished(StepId::new(1), false);
    assert_eq!(watch.offer(), None);
}

#[test]
fn clearing_forgets_the_offer() {
    let mut watch = MirrorWatch::default();
    started(&mut watch);
    watch.output(StepId::new(2), MISSING);
    watch.finished(StepId::new(2), false);

    watch.clear();

    assert_eq!(watch.offer(), None);
}
