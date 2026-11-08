use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use tokio::runtime::Runtime;
use toolbox_catalog::SkelCatalog;
use toolbox_core::features::maintenance::page::{CHAOTIC_MIRRORS, LaunchCause, ToolRow, tools};
use toolbox_plan::{Mirrorlist, PackageName, Plan, PlannedAction, RootAction};
use toolbox_system::mirrors::{MirrorRanking, RankProblem};

use super::{Maintenance, MaintenanceProblem, MaintenanceSetup};
use crate::launch::Launcher;
use crate::testing::poll;

const MIRRORS: &str = "Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n";

/// Remembers every start, or fails each one like a missing program.
#[derive(Debug, Default)]
struct Recorder {
    started: Mutex<Vec<PathBuf>>,
    missing: bool,
}

/// Answers every ranking the same.
struct Ranking(Result<&'static str, RankProblem>);

impl Launcher for Recorder {
    fn start(&self, program: &Path, _args: &[&str]) -> io::Result<()> {
        if self.missing {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }

        self.started.lock().push(program.to_owned());
        Ok(())
    }
}

impl MirrorRanking for Ranking {
    fn rank(&self) -> Result<Mirrorlist, RankProblem> {
        self.0.clone().map(|text| Mirrorlist::new(text).unwrap())
    }
}

fn maintenance(
    runtime: &Runtime,
    ranking: Result<&'static str, RankProblem>,
    home: &Path,
    skel_root: &Path,
) -> Maintenance {
    Maintenance::new(
        Some(MaintenanceSetup {
            runtime: runtime.handle().clone(),
            mirrors: Arc::new(Ranking(ranking)),
            home: home.to_owned(),
            skel_root: skel_root.to_owned(),
        }),
        egui::Context::default(),
    )
}

/// Takes results until one is a plan or the ranking and reset are done.
fn settle(maintenance: &mut Maintenance) -> Option<Plan> {
    poll("the job never ended", || {
        if let Some(plan) = maintenance.take() {
            return Some(Some(plan));
        }

        (!maintenance.ranking() && !maintenance.shelf().resetting()).then_some(None)
    })
}

fn installs(plan: &Plan) -> Vec<&str> {
    match plan.steps().first().map(|step| &step.action) {
        Some(PlannedAction::Root(RootAction::InstallPackages { packages })) => {
            packages.iter().map(PackageName::as_str).collect()
        }
        _ => Vec::new(),
    }
}

fn none_installed() -> Vec<ToolRow> {
    tools(|_| false)
}

fn all_installed() -> Vec<ToolRow> {
    tools(|_| true)
}

#[test]
fn a_missing_tool_is_installed_and_opens_once_the_install_ends_well() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    let launcher = Recorder::default();

    let plan = maintenance
        .open_tool("btrfs-assistant", &none_installed(), Some(&launcher))
        .expect("a missing tool is installed first");

    assert_eq!(installs(&plan), ["btrfs-assistant"]);
    assert!(launcher.started.lock().is_empty());

    maintenance.plan_started();
    maintenance.plan_finished(true, |_| true, Some(&launcher));

    assert_eq!(
        *launcher.started.lock(),
        [PathBuf::from("/usr/bin/btrfs-assistant-launcher")]
    );
}

#[test]
fn a_refused_or_failed_install_opens_nothing() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    let launcher = Recorder::default();

    maintenance.open_tool("btrfs-assistant", &none_installed(), Some(&launcher));
    maintenance.plan_refused();
    maintenance.plan_started();
    maintenance.plan_finished(true, |_| true, Some(&launcher));

    maintenance.open_tool("btrfs-assistant", &none_installed(), Some(&launcher));
    maintenance.plan_started();
    maintenance.plan_finished(false, |_| true, Some(&launcher));

    assert!(launcher.started.lock().is_empty());
}

#[test]
fn a_tool_that_cannot_start_says_why() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    let launcher = Recorder {
        missing: true,
        ..Recorder::default()
    };

    let plan = maintenance.open_tool("btrfs-assistant", &all_installed(), Some(&launcher));

    assert!(plan.is_none());
    assert!(matches!(
        maintenance.problem(),
        Some(MaintenanceProblem::Launch { tool, cause: LaunchCause::Missing, reason: _ })
            if *tool == "btrfs-assistant"
    ));

    maintenance.dismiss_problem();
    assert_eq!(maintenance.problem(), None);
}

#[test]
fn ranked_mirrors_are_installed_by_a_root_step() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));

    let plan = maintenance.open_tool(CHAOTIC_MIRRORS, &all_installed(), None);

    assert!(plan.is_none());
    assert!(maintenance.ranking());

    let ranked = settle(&mut maintenance).expect("the ranking ends in a plan");

    assert!(!maintenance.ranking());
    assert!(matches!(
        ranked.steps().first().map(|step| &step.action),
        Some(PlannedAction::Root(RootAction::InstallMirrorlist {
            mirrorlist: _
        }))
    ));
}

#[test]
fn a_failed_ranking_is_the_page_problem() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(
        &runtime,
        Err(RankProblem::TimedOut),
        Path::new("/"),
        Path::new("/"),
    );

    maintenance.open_tool(CHAOTIC_MIRRORS, &all_installed(), None);

    assert_eq!(settle(&mut maintenance), None);
    assert_eq!(
        maintenance.problem(),
        Some(&MaintenanceProblem::Mirrors(RankProblem::TimedOut))
    );
}

#[test]
fn a_missing_rate_mirrors_is_installed_and_then_ranks() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));

    let plan = maintenance
        .open_tool(CHAOTIC_MIRRORS, &none_installed(), None)
        .expect("rate-mirrors is installed first");

    assert_eq!(installs(&plan), ["rate-mirrors"]);
    assert!(!maintenance.ranking());

    maintenance.plan_started();
    maintenance.plan_finished(true, |_| true, None);

    assert!(maintenance.ranking());
    assert!(settle(&mut maintenance).is_some());
}

#[test]
fn a_reset_copies_the_chosen_settings_into_the_home() {
    let runtime = Runtime::new().unwrap();
    let skel = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(skel.path().join(".config")).unwrap();
    std::fs::write(skel.path().join(".config/kitty.conf"), "font_size 12\n").unwrap();

    let catalog = SkelCatalog::parse(
        "[[config]]\nid = \"kitty\"\nname = \"Kitty\"\nfiles = [\"/etc/skel/.config/kitty.conf\"]\n",
        "skel.toml",
    )
    .unwrap();

    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), home.path(), skel.path());
    maintenance.stock(Some(&catalog));
    maintenance.choose("kitty", true);
    maintenance.reset(Some(&catalog));

    assert!(maintenance.shelf().resetting());
    assert_eq!(settle(&mut maintenance), None);
    assert_eq!(maintenance.shelf().outcome().copied, 1);
    assert_eq!(
        std::fs::read_to_string(home.path().join(".config/kitty.conf")).unwrap(),
        "font_size 12\n"
    );
}

#[test]
fn the_dry_run_ranks_a_fixed_list_at_once() {
    let mirrorlist = super::SampleMirrors.rank().unwrap();

    assert!(mirrorlist.as_str().contains("Server = https://"));
}
