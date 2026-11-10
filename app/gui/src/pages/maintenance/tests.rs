mod common;
mod repairs;
mod reset;

use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::maintenance::page::LaunchCause;
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::Toggle;
use toolbox_system::mirrors::RankProblem;
use toolbox_system::skel::shelf::{Outcome, SkelRow};

use super::MaintenancePage;
use crate::command::{Command, MaintenanceCommand};
use crate::maintenance::{Maintenance, MaintenanceProblem};
use crate::programs::Programs;
use crate::testing::{PLAN_RUNNING, PageFixture, PageState, button_kind};

type Page = Harness<'static, PageState<MaintenancePage>>;

fn programs() -> Programs {
    Programs::found(|path| {
        path != Path::new("/usr/bin/btrfs-assistant-launcher")
            && path != Path::new("/usr/bin/rate-mirrors")
    })
}

fn kitty() -> SkelRow {
    SkelRow {
        entry: "kitty".to_owned(),
        name: "Kitty".to_owned(),
        files: vec!["/etc/skel/.config/kitty/kitty.conf".to_owned()],
        targets: vec!["~/.config/kitty/kitty.conf".to_owned()],
    }
}

fn page(fixture: PageFixture) -> Page {
    let mut harness = fixture.harness(MaintenancePage::default());
    harness.run();
    harness
}

fn with(programs: Programs, maintenance: Maintenance) -> PageFixture {
    PageFixture {
        programs: Some(programs),
        maintenance,
        ..PageFixture::default()
    }
}

fn sent(harness: &Page) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn open_tab(harness: &mut Page, tab: &str) {
    harness
        .get_by_role_and_label(Role::RadioButton, tab)
        .click();
    harness.run();
}

fn disabled(harness: &Page, label: &str) -> Vec<bool> {
    harness
        .query_all_by_label(label)
        .map(|node| node.accesskit_node().is_disabled())
        .collect()
}

#[test]
fn an_operation_runs_now_or_waits_in_the_queue() {
    let mut harness = page(with(programs(), Maintenance::default()));

    assert!(harness.query_by_label("Queue removal").is_none());
    harness
        .get_by_label("More actions for Remove orphans")
        .click();
    harness.run();
    harness.get_by_label("Queue removal").click();
    harness.run();
    harness.get_all_by_label("Run now").next().unwrap().click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            Command::from(MaintenanceCommand::Queue {
                operation: "remove-orphans".to_owned(),
                on: true,
            }),
            Command::from(MaintenanceCommand::RunNow("upgrade-system".to_owned())),
        ]
    );
}

#[test]
fn a_missing_tool_offers_its_install_and_the_ranking_says_it_runs() {
    let ranking = Maintenance::showing(None, true, Vec::new(), None);
    let mut harness = page(with(programs(), ranking));

    assert_eq!(disabled(&harness, "Install and open…"), [true, false]);
    assert!(
        harness
            .query_by_label_contains("Testing the mirrors")
            .is_some()
    );

    harness
        .get_all_by_label("Install and open…")
        .nth(1)
        .unwrap()
        .scroll_to_me();
    harness.run();
    harness
        .get_all_by_label("Install and open…")
        .nth(1)
        .unwrap()
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::OpenTool(
            "btrfs-assistant".to_owned()
        ))]
    );
}

#[test]
fn a_running_plan_rests_every_button_that_starts_one() {
    let harness = page(PageFixture {
        busy: true,
        ..with(programs(), Maintenance::default())
    });

    assert!(harness.query_by_label(PLAN_RUNNING).is_some());
    assert!(disabled(&harness, "Run now").iter().all(|rests| *rests));
    assert!(disabled(&harness, "Open").iter().all(|rests| *rests));
}

#[test]
fn a_tool_that_did_not_start_says_why_until_dismissed() {
    let problem = MaintenanceProblem::Launch {
        tool: "btrfs-assistant",
        cause: LaunchCause::Missing,
        reason: "No such file".to_owned(),
    };

    let maintenance = Maintenance::showing(Some(problem), false, Vec::new(), None);
    let mut harness = page(with(programs(), maintenance));

    assert!(
        harness
            .query_by_label_contains(
                "Btrfs Assistant could not be started because it is not installed."
            )
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::DismissProblem)]
    );
}

#[test]
fn a_failed_ranking_names_the_system_reason() {
    let problem = MaintenanceProblem::Mirrors(RankProblem::NotStarted("denied".to_owned()));
    let maintenance = Maintenance::showing(Some(problem), false, Vec::new(), None);
    let harness = page(with(programs(), maintenance));

    assert!(
        harness
            .query_by_label_contains("rate-mirrors could not be started")
            .is_some()
    );
    assert!(harness.query_by_label_contains(": denied").is_some());
}

#[test]
fn repairs_lists_the_repairs_it_can_run() {
    let mut harness = page(with(programs(), Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    assert!(harness.query_by_label("Reset audio").is_some());
    assert!(harness.query_by_label("Install garuda-update").is_none());

    harness.get_all_by_label("Run").next().unwrap().click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::RunNow(
            "health-check".to_owned()
        ))]
    );
}

#[test]
fn without_garuda_update_repairs_offers_it() {
    let programs = Programs::found(|path| path == Path::new("/usr/bin/garuda-health"));
    let mut harness = page(with(programs, Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    assert!(harness.query_by_label("System health check").is_some());

    harness.get_by_label("Install garuda-update").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::InstallGarudaUpdate)]
    );
}

#[test]
fn a_system_without_defaults_says_so() {
    let mut harness = page(with(programs(), Maintenance::default()));
    open_tab(&mut harness, "Reset settings");

    assert!(
        harness
            .query_by_label("No default settings found")
            .is_some()
    );
}

#[test]
fn a_chosen_setting_is_reset_after_the_question_lists_its_files() {
    let mut maintenance = Maintenance::showing(None, false, vec![kitty()], None);
    let mut harness = page(with(
        programs(),
        Maintenance::showing(None, false, vec![kitty()], None),
    ));

    open_tab(&mut harness, "Reset settings");

    harness
        .get_by_role_and_label(Role::CheckBox, "Kitty")
        .click();
    harness.run();
    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::Choose {
            entry: "kitty".to_owned(),
            on: true,
        })]
    );

    maintenance.choose("kitty", Toggle::On);
    harness.state_mut().fixture.maintenance = maintenance;
    harness.run();
    harness.get_by_label("Reset selected…").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Reset 1 setting?")
            .is_some()
    );
    assert!(
        harness
            .query_by_label("~/.config/kitty/kitty.conf")
            .is_some()
    );

    harness.get_by_label("Reset").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(MaintenanceCommand::Reset)]);
    assert!(harness.query_by_role(Role::Dialog).is_none());
}

#[test]
fn nothing_chosen_leaves_the_reset_resting() {
    let maintenance = Maintenance::showing(None, false, vec![kitty()], None);
    let mut harness = page(with(programs(), maintenance));
    open_tab(&mut harness, "Reset settings");

    assert_eq!(disabled(&harness, "Reset selected…"), [true]);
}

#[test]
fn a_reset_that_failed_names_the_applications() {
    let outcome = Outcome {
        done: true,
        copied: 0,
        failed: vec!["Kitty".to_owned()],
    };

    let maintenance = Maintenance::showing(None, false, vec![kitty()], Some(outcome));
    let mut harness = page(with(programs(), maintenance));
    open_tab(&mut harness, "Reset settings");

    assert!(
        harness
            .query_by_label_contains("Some settings could not be reset: Kitty")
            .is_some()
    );
}

fn kinds(harness: &Page, label: &str) -> Vec<ButtonKind> {
    harness
        .query_all_by_label(label)
        .map(|node| crate::testing::kind_at(harness, node.rect().center()))
        .collect()
}

#[test]
fn only_the_system_update_leads_the_common_tab() {
    let harness = page(with(programs(), Maintenance::default()));
    let runs = kinds(&harness, "Run now");

    assert_eq!(runs.first(), Some(&ButtonKind::Primary));
    assert!(
        runs.iter().skip(1).all(|kind| *kind == ButtonKind::Soft),
        "{runs:?}"
    );
}

#[test]
fn the_reset_question_answers_with_danger() {
    let mut maintenance = Maintenance::showing(None, false, vec![kitty()], None);
    maintenance.choose("kitty", Toggle::On);
    let mut harness = page(with(programs(), maintenance));
    open_tab(&mut harness, "Reset settings");
    harness.get_by_label("Reset selected…").click();
    harness.run();

    assert_eq!(button_kind(&harness, "Reset"), ButtonKind::Danger);
}
