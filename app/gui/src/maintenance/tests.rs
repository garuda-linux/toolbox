mod after_install;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use parking_lot::Mutex;
use tokio::runtime::Runtime;
use toolbox_catalog::SkelCatalog;
use toolbox_core::features::maintenance::Tool;
use toolbox_core::features::maintenance::page::{PageTool, ToolRow, tools};
use toolbox_plan::{Mirrorlist, PackageName, Plan, PlannedAction, RootAction, Toggle};
use toolbox_system::mirrors::{MirrorRanking, RankProblem};

use super::{Maintenance, MaintenanceProblem, MaintenanceSetup};
use crate::catalogs::Catalogs;
use crate::command::{AppCommand, MaintenanceCommand};
use crate::launch::Launcher;
use crate::testing::poll;

const MIRRORS: &str = "Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n";

/// Remembers every start.
#[derive(Debug, Default)]
struct Recorder {
    started: Mutex<Vec<PathBuf>>,
}

/// Answers every ranking the same.
struct Ranking(Result<&'static str, RankProblem>);

impl Launcher for Recorder {
    fn start(&self, program: &Path, _args: &[&str]) {
        self.started.lock().push(program.to_owned());
    }
}

impl MirrorRanking for Ranking {
    fn rank(&self, _cancel: &Arc<AtomicBool>) -> Result<Mirrorlist, RankProblem> {
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

/// The recorder as the launcher the page holds.
fn as_launcher(launcher: &Arc<Recorder>) -> Arc<dyn Launcher> {
    let shared: Arc<Recorder> = Arc::clone(launcher);
    shared
}

/// Waits for the disk check of a finished install, which lands off the
/// frame, and acts on it.
pub(super) fn landed(maintenance: &mut Maintenance) -> Option<Plan> {
    poll("the disk check never landed", || {
        if let Some(plan) = maintenance.take() {
            return Some(Some(plan));
        }

        (!maintenance.checking()).then_some(None)
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
    let launcher = Arc::new(Recorder::default());

    let plan = maintenance
        .open_tool(
            PageTool::Program(Tool::BtrfsAssistant),
            &none_installed(),
            Some(&*launcher),
        )
        .expect("a missing tool is installed first");

    assert_eq!(installs(&plan), ["btrfs-assistant"]);
    assert!(launcher.started.lock().is_empty());

    maintenance.plan_started();
    maintenance.plan_finished(true, |_| true, Some(as_launcher(&launcher)));
    landed(&mut maintenance);

    assert_eq!(
        *launcher.started.lock(),
        [PathBuf::from("/usr/bin/btrfs-assistant-launcher")]
    );
}

#[test]
fn a_refused_or_failed_install_opens_nothing() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    let launcher = Arc::new(Recorder::default());

    maintenance.open_tool(
        PageTool::Program(Tool::BtrfsAssistant),
        &none_installed(),
        Some(&*launcher),
    );
    maintenance.plan_refused();
    maintenance.plan_started();
    maintenance.plan_finished(true, |_| true, Some(as_launcher(&launcher)));
    landed(&mut maintenance);

    maintenance.open_tool(
        PageTool::Program(Tool::BtrfsAssistant),
        &none_installed(),
        Some(&*launcher),
    );
    maintenance.plan_started();
    maintenance.plan_finished(false, |_| true, Some(as_launcher(&launcher)));
    landed(&mut maintenance);

    assert!(launcher.started.lock().is_empty());
}

/// A failed start is a toast from the launcher, so the page keeps no
/// problem for it.
#[test]
fn a_tool_is_handed_to_the_launcher_and_keeps_no_problem() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    let launcher = Arc::new(Recorder::default());

    let plan = maintenance.open_tool(
        PageTool::Program(Tool::BtrfsAssistant),
        &all_installed(),
        Some(&*launcher),
    );

    assert!(plan.is_none());
    assert_eq!(launcher.started.lock().len(), 1);
    assert_eq!(maintenance.problem(), None);
}

#[test]
fn ranked_mirrors_are_installed_by_a_root_step() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));

    let plan = maintenance.open_tool(PageTool::ChaoticMirrors, &all_installed(), None);

    assert!(plan.is_none());
    let ranked = settle(&mut maintenance).expect("the ranking ends in a plan");

    assert!(!maintenance.ranking());
    assert!(matches!(
        ranked.steps().first().map(|step| &step.action),
        Some(PlannedAction::Root(RootAction::InstallMirrorlist {
            mirrorlist: _
        }))
    ));
}

/// The ranking landed while an update ran, so the Console refused it.
#[test]
fn a_refused_ranked_list_goes_out_after_the_running_plan() {
    let runtime = Runtime::new().unwrap();
    let mut maintenance = maintenance(&runtime, Ok(MIRRORS), Path::new("/"), Path::new("/"));
    maintenance.open_tool(PageTool::ChaoticMirrors, &all_installed(), None);
    let ranked = settle(&mut maintenance).expect("the ranking ends in a plan");

    maintenance.mirrorlist_refused();
    assert_eq!(maintenance.take(), None, "the update still runs");

    maintenance.plan_finished(true, |_| true, None);
    assert_eq!(landed(&mut maintenance), Some(ranked));

    maintenance.mirrorlist_started();
    maintenance.plan_finished(true, |_| true, None);
    assert_eq!(landed(&mut maintenance), None, "installed once");
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

    maintenance.open_tool(PageTool::ChaoticMirrors, &all_installed(), None);

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
        .open_tool(PageTool::ChaoticMirrors, &none_installed(), None)
        .expect("rate-mirrors is installed first");

    assert_eq!(installs(&plan), ["rate-mirrors"]);
    assert!(!maintenance.ranking());

    maintenance.plan_started();
    maintenance.plan_finished(true, |_| true, None);

    let ranked = landed(&mut maintenance).or_else(|| settle(&mut maintenance));
    assert!(ranked.is_some(), "the ranking ends in a plan");
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
    maintenance.choose("kitty", Toggle::On);
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
    let mirrorlist = super::SampleMirrors.rank(&Arc::default()).unwrap();

    assert!(mirrorlist.as_str().contains("Server = https://"));
}

#[test]
fn a_dismissed_problem_and_a_chosen_entry_are_the_services() {
    let mut maintenance = Maintenance::default();

    for command in [
        MaintenanceCommand::DismissProblem,
        MaintenanceCommand::Choose {
            entry: "kitty".to_owned(),
            on: true,
        },
    ] {
        let handled = maintenance.handle(&AppCommand::Maintenance(command));

        assert!(handled.taken);
        assert!(handled.edit.is_none());
    }

    assert_eq!(maintenance.problem(), None);
}

/// The service holds the catalogs, so the reset is its own click.
#[test]
fn a_reset_click_copies_what_the_shared_skel_catalog_names() {
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
    maintenance.set_catalogs(Arc::new(Catalogs {
        skel: Some(catalog),
        ..Catalogs::default()
    }));
    poll("the offer never landed", || {
        maintenance.take();
        (!maintenance.shelf().rows().is_empty()).then_some(())
    });
    maintenance.choose("kitty", Toggle::On);

    let handled = maintenance.handle(&AppCommand::Maintenance(MaintenanceCommand::Reset));

    assert!(handled.taken);
    assert_eq!(settle(&mut maintenance), None);
    assert_eq!(maintenance.shelf().outcome().copied, 1);
}
