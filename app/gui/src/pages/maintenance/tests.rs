mod common;
mod repairs;
mod reset;

use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::maintenance::page::{PageTool, Repair};
use toolbox_core::features::maintenance::{Operation, Tool};
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::Toggle;
use toolbox_system::mirrors::RankProblem;
use toolbox_system::skel::shelf::{Outcome, SkelRow};

use super::MaintenancePage;
use crate::command::{Command, MaintenanceCommand};
use crate::maintenance::{Maintenance, MaintenanceProblem};
use crate::page_service::Services;
use crate::programs::Programs;
use crate::testing::{PLAN_RUNNING, PageFixture, PageState, button_kind, paints_icon, rect_of};

type Page = Harness<'static, PageState<MaintenancePage>>;

fn programs() -> Programs {
    Programs::found(|path| {
        path != Path::new("/usr/bin/btrfs-assistant") && path != Path::new("/usr/bin/rate-mirrors")
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

/// A page whose programs are not read yet. Read again spins while they are
/// read, so its frames never settle.
fn pending(fixture: PageFixture) -> Page {
    let mut harness = fixture.harness(MaintenancePage::default());
    harness.run_steps(3);
    harness
}

fn with(programs: Programs, maintenance: Maintenance) -> PageFixture {
    PageFixture {
        programs: Some(programs),
        services: Services {
            maintenance,
            ..Services::default()
        },
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

    // Read again spins while the programs of a fixture are not read.
    harness.run_steps(3);
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

    assert!(
        harness
            .query_by_label("More actions for Remove orphans")
            .is_none()
    );
    harness.get_by_label("Queue Removal").click();
    harness.run();
    harness.get_all_by_label("Run Now").next().unwrap().click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            Command::from(MaintenanceCommand::Queue {
                operation: Operation::RemoveOrphans,
                on: true,
            }),
            Command::from(MaintenanceCommand::RunOperation(Operation::UpgradeSystem)),
        ]
    );
}

#[test]
fn a_missing_tool_offers_its_install_and_the_ranking_says_it_runs() {
    let ranking = Maintenance::showing(None, true, Vec::new(), None);
    let mut harness = page(with(programs(), ranking));

    assert_eq!(disabled(&harness, "Install and Open…"), [false]);
    assert!(
        harness
            .query_by_label_contains("Testing the mirrors")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::ProgressIndicator, "Ranking mirrors, 0:00")
            .is_some()
    );

    harness.get_by_label("Install and Open…").scroll_to_me();
    harness.run();
    harness.get_by_label("Install and Open…").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::OpenTool(
            PageTool::Program(Tool::BtrfsAssistant)
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
    let run_now = disabled(&harness, "Run Now");
    let open = disabled(&harness, "Open");
    assert!(
        !run_now.is_empty() && !open.is_empty(),
        "no buttons to check"
    );
    assert!(run_now.iter().all(|rests| *rests));
    assert!(open.iter().all(|rests| *rests));
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

    assert!(harness.query_by_label("Reset Audio").is_some());
    assert!(harness.query_by_label("Install garuda-update").is_none());

    harness.get_all_by_label("Run").next().unwrap().click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::RunRepair(
            Repair::HealthCheck
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
        [Command::from(MaintenanceCommand::InstallMissing)]
    );
}

#[test]
fn without_garuda_health_alone_one_banner_offers_it() {
    let programs = Programs::found(|path| path != Path::new("/usr/bin/garuda-health"));
    let mut harness = page(with(programs, Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    assert!(harness.query_by_label("Install garuda-health").is_some());
}

#[test]
fn one_banner_names_and_installs_every_missing_package() {
    let mut harness = page(with(Programs::found(|_| false), Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    assert!(
        harness
            .query_by_label_contains("need garuda-health, garuda-update")
            .is_some()
    );

    harness
        .get_by_label("Install garuda-health, garuda-update")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::InstallMissing)]
    );
}

#[test]
fn a_system_without_defaults_says_so() {
    let mut harness = page(with(programs(), Maintenance::default()));
    open_tab(&mut harness, "Reset Settings");

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

    open_tab(&mut harness, "Reset Settings");

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
    harness.state_mut().fixture.services.maintenance = maintenance;
    harness.run();
    harness.get_by_label("Reset Selected…").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Reset 1 Setting?")
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
    open_tab(&mut harness, "Reset Settings");

    assert_eq!(disabled(&harness, "Reset Selected…"), [true]);
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
    open_tab(&mut harness, "Reset Settings");

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
    let runs = kinds(&harness, "Run Now");

    assert_eq!(runs.first(), Some(&ButtonKind::Primary));
    assert!(
        runs.iter().skip(1).all(|kind| *kind == ButtonKind::Soft),
        "{runs:?}"
    );
}

#[test]
fn the_reset_question_answers_without_a_red_fill() {
    let mut maintenance = Maintenance::showing(None, false, vec![kitty()], None);
    maintenance.choose("kitty", Toggle::On);
    let mut harness = page(with(programs(), maintenance));
    open_tab(&mut harness, "Reset Settings");
    harness.get_by_label("Reset Selected…").click();
    harness.run();

    // Danger draws as Soft (buttons.rs), so only its delete icon marks it.
    assert_eq!(button_kind(&harness, "Reset"), ButtonKind::Soft);
    let answer = rect_of(&harness, Role::Button, "Reset");
    assert!(paints_icon(&harness, answer, icons::EDIT_DELETE));
}

#[test]
fn the_tool_buttons_share_one_width_and_left_edge() {
    let ranking = Maintenance::showing(None, true, Vec::new(), None);
    let harness = page(with(programs(), ranking));

    let buttons: Vec<egui::Rect> = ["Install and Open…", "Open"]
        .into_iter()
        .flat_map(|label| harness.query_all_by_label(label).map(|node| node.rect()))
        .collect();

    let first = buttons.first().expect("a tool button").width();

    assert!(buttons.len() > 1, "more than one tool button");
    for button in &buttons {
        assert!(
            (button.width() - first).abs() < 0.5,
            "a tool button is {} wide, the first {first}",
            button.width()
        );
    }

    let left = buttons.iter().map(egui::Rect::left).collect::<Vec<_>>();
    assert!(
        left.iter().all(|edge| (edge - left[0]).abs() < 0.5),
        "left edges {left:?}"
    );
}
