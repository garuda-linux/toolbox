//! The removal guards read the dependency graph from every snapshot. A
//! failed read leaves it out, so the guards refuse instead of guessing.

use toolbox_state::packages::PackageGraph;
use toolbox_state::probes::Probe;

use super::fakes::{FakePackages, sources};
use super::{host, installed};
use crate::{ProbeError, probe_snapshot};

#[test]
fn the_package_graph_lands_in_the_snapshot() {
    let (snapshot, _issues) = probe_snapshot(&sources(FakePackages(Ok(installed())), host()));

    assert!(snapshot.part::<PackageGraph>().is_some());
}

#[test]
fn a_failed_graph_read_is_reported_and_left_out() {
    let failed = FakePackages(Err(ProbeError::Failed("database locked".to_owned())));

    let (snapshot, issues) = probe_snapshot(&sources(failed, host()));

    assert!(snapshot.part::<PackageGraph>().is_none());
    assert!(
        issues
            .iter()
            .any(|issue| issue.probe == Probe::PackageGraph)
    );
}
