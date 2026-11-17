//! What follows an install the page sent: a program that is still not
//! there says so, and a ranking under way is never started twice.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Runtime;
use toolbox_core::features::maintenance::Tool;
use toolbox_core::features::maintenance::page::{LaunchCause, PageTool};
use toolbox_plan::Mirrorlist;
use toolbox_system::mirrors::{MirrorRanking, RankProblem};

use super::{
    MIRRORS, Recorder, all_installed, as_launcher, landed, maintenance, none_installed, settle,
};
use crate::maintenance::{Maintenance, MaintenanceProblem, MaintenanceSetup};

/// Counts its rankings, each of which answers at once.
#[derive(Debug, Default)]
struct Counted(Arc<AtomicUsize>);

impl MirrorRanking for Counted {
    fn rank(&self, _cancel: &Arc<AtomicBool>) -> Result<Mirrorlist, RankProblem> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(Mirrorlist::new(MIRRORS).unwrap())
    }
}

#[test]
fn a_tool_still_missing_after_its_install_says_so() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    let launcher = Arc::new(Recorder::default());
    maintenance.open_tool(
        PageTool::Program(Tool::BtrfsAssistant),
        &none_installed(),
        Some(&*launcher),
    );

    maintenance.plan_started();
    maintenance.plan_finished(true, |_| false, Some(as_launcher(&launcher)));
    landed(&mut maintenance);

    assert!(launcher.started.lock().is_empty());
    assert_eq!(
        maintenance.problem(),
        Some(&MaintenanceProblem::Launch {
            tool: "btrfs-assistant",
            cause: LaunchCause::AbsentAfterInstall,
            reason: "/usr/bin/btrfs-assistant".to_owned(),
        })
    );
}

#[test]
fn rate_mirrors_still_missing_after_its_install_says_so() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    maintenance.open_tool(PageTool::ChaoticMirrors, &none_installed(), None);

    maintenance.plan_started();
    maintenance.plan_finished(true, |_| false, None);
    landed(&mut maintenance);

    assert!(!maintenance.ranking());
    assert!(matches!(
        maintenance.problem(),
        Some(MaintenanceProblem::Launch {
            tool: "chaotic-mirrors",
            cause: LaunchCause::AbsentAfterInstall,
            reason: _,
        })
    ));
}

#[test]
fn a_second_ranking_waits_for_the_one_under_way() {
    let runtime = Runtime::new().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let mut maintenance = Maintenance::new(
        Some(MaintenanceSetup {
            runtime: runtime.handle().clone(),
            mirrors: Arc::new(Counted(Arc::clone(&count))),
            home: "/".into(),
            skel_root: "/".into(),
        }),
        egui::Context::default(),
    );

    maintenance.open_tool(PageTool::ChaoticMirrors, &all_installed(), None);
    maintenance.open_tool(PageTool::ChaoticMirrors, &all_installed(), None);
    assert!(settle(&mut maintenance).is_some());
    drop(maintenance);

    // Waits for every ranking that was started, a second one included.
    runtime.shutdown_timeout(Duration::from_secs(5));

    assert_eq!(count.load(Ordering::SeqCst), 1);
}
