//! The removal guards' read: the installed packages' dependency graph,
//! from the captured local database.

use toolbox_state::packages::PackageGraph;

use super::fixture::{GEGL, GIMP, MPV, System, YT_DLP};
use crate::ProbeError;
use crate::packages::{Libalpm, PackageIndexSource as _, PackageSource as _};

#[test]
fn the_graph_holds_the_same_nodes_as_the_index() {
    let system = System::new(&["core"]);
    system.install_captured(&[GIMP, GEGL, MPV, YT_DLP]);
    system.sync("core", &["gimp"]);
    let alpm = Libalpm::with_config(system.config());

    assert_eq!(alpm.graph(), Ok(PackageGraph(alpm.index().unwrap().local)));
}

#[test]
fn the_graph_needs_no_synced_database() {
    let system = System::new(&["core"]);
    system.install_captured(&[GIMP, GEGL]);

    let graph = Libalpm::with_config(system.config()).graph().unwrap();

    let gimp = graph.0.iter().find(|node| node.name.as_str() == "gimp");
    assert!(gimp.is_some_and(|node| node.depends.contains(&"gegl".to_owned())));
}

#[test]
fn without_a_local_database_there_is_no_graph() {
    let system = System::new(&["core"]);

    assert_eq!(
        Libalpm::with_config(system.config()).graph(),
        Err(ProbeError::Unavailable)
    );
}
