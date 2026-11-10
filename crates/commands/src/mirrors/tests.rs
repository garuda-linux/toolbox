use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_plan::{Mirrorlist, Plan, PlannedAction, RootAction};
use toolbox_system::mirrors::{MirrorRanking, RankProblem};

use super::steps;
use crate::out::{Out, Settings};

/// The list `rate-mirrors` saved for Chaotic-AUR.
const RANKED: &str = toolbox_plan::testing::mirrorlist::RATE_MIRRORS_CHAOTIC;

struct Ranked;

impl MirrorRanking for Ranked {
    fn rank(&self) -> Result<Mirrorlist, RankProblem> {
        Ok(Mirrorlist::new(RANKED.to_owned()).unwrap())
    }
}

struct Offline;

impl MirrorRanking for Offline {
    fn rank(&self) -> Result<Mirrorlist, RankProblem> {
        Err(RankProblem::Failed(
            "exit status: 1: no mirror answered".to_owned(),
        ))
    }
}

fn kinds(plans: &[Plan]) -> Vec<&'static str> {
    plans
        .iter()
        .flat_map(Plan::steps)
        .map(|step| match &step.action {
            PlannedAction::Root(RootAction::InstallPackages { packages: _ }) => "install-packages",
            PlannedAction::Root(RootAction::InstallMirrorlist { mirrorlist: _ }) => {
                "install-mirrorlist"
            }
            PlannedAction::Root(_) | PlannedAction::User(_) => "other",
        })
        .collect()
}

fn ranked_with(
    installed: bool,
    confirm: Confirm,
    ranking: &dyn MirrorRanking,
) -> (Exit, Vec<Plan>) {
    let mut sink = Vec::new();
    let mut out = Out::new(&mut sink, Settings::default());
    let mut plans = Vec::new();

    let exit = steps(installed, confirm, ranking, &mut out, |plan, _out| {
        plans.push(plan);
        Exit::Success
    });

    (exit, plans)
}

#[test]
fn the_ranked_list_is_installed_as_root() {
    let (exit, plans) = ranked_with(true, Confirm::default(), &Ranked);

    assert_eq!(exit, Exit::Success);
    assert_eq!(kinds(&plans), ["install-mirrorlist"]);
}

/// As the window: without `rate-mirrors` its package comes first.
#[test]
fn a_missing_ranking_tool_is_installed_first() {
    let (exit, plans) = ranked_with(false, Confirm::default(), &Ranked);

    assert_eq!(exit, Exit::Success);
    assert_eq!(kinds(&plans), ["install-packages", "install-mirrorlist"]);
}

/// A dry run cannot rank with a tool it did not install.
#[test]
fn a_dry_run_without_the_tool_stops_after_its_install() {
    let confirm = Confirm {
        dry_run: true,
        ..Confirm::default()
    };

    let (_, plans) = ranked_with(false, confirm, &Ranked);

    assert_eq!(kinds(&plans), ["install-packages"]);
}

#[test]
fn a_failed_ranking_installs_nothing() {
    let (exit, plans) = ranked_with(true, Confirm::default(), &Offline);

    assert_eq!(exit, Exit::Failure);
    assert_eq!(plans, []);
}
