use std::collections::BTreeSet;

use toolbox_aur::{AurError, AurSource, DownloadError, RequestError};
use toolbox_cli::Exit;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::{AurIndex, AurPackage};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{PackageName, Plan, PlannedAction, UserAction};
use toolbox_services::aur_orders::RouteProblem;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;
use toolbox_system::testing::aur_orders::FakeAurOrders;

use super::{
    aur_info, aur_info_text, aur_listing, found, install_onto, plan_install, waiting_text,
};
use crate::change::{Refused, fresh};

/// paru-bin, paru and yay-bin, paru the least popular.
struct Aur(Option<AurError>);

impl AurSource for Aur {
    fn index(&self) -> Result<AurIndex, AurError> {
        if let Some(error) = &self.0 {
            return Err(error.clone());
        }

        Ok(AurIndex {
            packages: vec![
                package("yay-bin", 5.0),
                package("paru", 1.0),
                package("paru-bin", 9.0),
            ],
            downloaded: 1_736_467_200,
            stale: false,
        })
    }
}

/// The AUR's copy with one package, octopi-git, that lists `conflicts`.
struct Copy(Vec<AurPackage>);

impl AurSource for Copy {
    fn index(&self) -> Result<AurIndex, AurError> {
        Ok(AurIndex {
            packages: self.0.clone(),
            downloaded: 1_736_467_200,
            stale: false,
        })
    }
}

fn octopi_copy(conflicts: &[&str]) -> Copy {
    let mut octopi = package("octopi-git", 3.0);
    octopi.conflicts = conflicts
        .iter()
        .map(|name| toolbox_text::plain(name))
        .collect();

    Copy(vec![octopi])
}
fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn package(text: &str, popularity: f64) -> AurPackage {
    AurPackage {
        name: name(text),
        version: toolbox_text::plain("1.0-1"),
        description: toolbox_text::plain(&format!("About {text}")),
        url: Some(toolbox_text::plain("https://example.org")),
        votes: 7,
        popularity,
        out_of_date: None,
        maintainer: Some(toolbox_text::plain("someone")),
        first_submitted: 1_736_467_200,
        last_modified: 1_736_467_200,
        depends: vec![toolbox_text::plain("glibc")],
        make_depends: vec![toolbox_text::plain("cargo")],
        opt_depends: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        licenses: vec![toolbox_text::plain("MIT")],
        keywords: Vec::new(),
    }
}

fn install(packages: &[PackageName], aur: &Aur, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| install_onto(packages, aur, snapshot, changes))
}

fn installed() -> BTreeSet<PackageName> {
    BTreeSet::from([name("paru-bin")])
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(installed()));
    snapshot.insert(refused_route());
    snapshot
}

/// The helper refuses this user's build folder, so builds take paru's way.
fn refused_route() -> AurRoute {
    AurRoute::Refused {
        kind: RouteKind::Linked,
        detail: "a folder on the way is missing".to_owned(),
    }
}

/// The helper accepts this user's build folder.
fn accepted_route() -> AurRoute {
    AurRoute::Accepted
}

#[test]
fn an_aur_search_puts_the_exact_name_first_and_marks_installed_ones() {
    let rows = found(Some("paru"), &Aur(None), &installed()).unwrap();

    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(names, ["paru", "paru-bin"]);
    assert!(rows[1].installed);
    assert!(!rows[0].installed);

    let all = found(None, &Aur(None), &installed()).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(aur_listing(&all).summary.as_deref(), Some("3, 1 installed"));
}

#[test]
fn an_aur_search_without_a_copy_says_why() {
    let error = found(
        None,
        &Aur(Some(AurError::Waiting { next: 1 })),
        &installed(),
    );

    assert_eq!(error.unwrap_err(), AurError::Waiting { next: 1 });
}

#[test]
fn an_aur_package_shows_its_facts_and_its_page() {
    let info = aur_info(&name("paru"), &Aur(None), &installed())
        .unwrap()
        .unwrap();

    assert_eq!(info.aur_page, "https://aur.archlinux.org/packages/paru");
    assert!(!info.installed);
    let text = aur_info_text(crate::out::paint(crate::out::Settings::default()), &info);
    for fact in ["2025-01-10", "cargo", "MIT", "someone", "votes"] {
        assert!(text.contains(fact), "{fact} in {text}");
    }

    assert_eq!(
        aur_info(&name("pacman"), &Aur(None), &installed()).unwrap(),
        None
    );
}

#[test]
fn an_aur_install_plans_one_paru_build_for_every_name() {
    let plan = install(&[name("yay-bin"), name("paru")], &Aur(None), &snapshot()).unwrap();

    assert!(matches!(
        &plan.steps()[0].action,
        PlannedAction::User(UserAction::InstallAur { packages })
            if packages.iter().map(PackageName::as_str).eq(["paru", "yay-bin"])
    ));
}

#[test]
fn an_aur_install_refuses_an_unknown_and_an_installed_package() {
    let mut changes = ChangeSet::default();
    let unknown = install_onto(&[name("parru")], &Aur(None), &snapshot(), &mut changes);
    assert!(
        matches!(&unknown, Err(Refused::Unknown(message)) if message.contains("paru")),
        "{unknown:?}"
    );

    assert!(matches!(
        install(&[name("paru-bin")], &Aur(None), &snapshot()),
        Err(Refused::NothingToDo(_))
    ));

    assert!(matches!(
        install(
            &[name("paru")],
            &Aur(Some(AurError::Download(DownloadError::Request(
                RequestError::NoAnswer("offline".to_owned()),
            )))),
            &snapshot()
        ),
        Err(Refused::Unavailable(_))
    ));
}

#[test]
fn a_waiting_copy_names_the_day_of_the_next_download() {
    assert!(waiting_text(1_736_467_200).contains("2025-01-10"));
}

/// paru's order of octopi-git, as its captured `paru -Po` answered.
fn octopi_orders() -> FakeAurOrders {
    FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))])
}

fn snapshot_with(route: AurRoute) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(installed()));
    snapshot.insert(route);
    snapshot
}

fn split_plan(route: AurRoute, copy: &Copy, orders: &FakeAurOrders) -> Result<Plan, Refused> {
    plan_install(
        &[name("octopi-git")],
        copy,
        &snapshot_with(route),
        orders,
        Ok(()),
    )
}

#[test]
fn an_accepted_route_plans_the_review_first_and_then_the_octopi_builds() {
    let plan = split_plan(accepted_route(), &octopi_copy(&[]), &octopi_orders()).unwrap();
    let actions: Vec<&PlannedAction> = plan.steps().iter().map(|step| &step.action).collect();

    assert!(matches!(
        actions.first(),
        Some(PlannedAction::User(UserAction::ReviewAur { bases }))
            if bases.iter().map(PackageName::as_str).eq(["alpm_octopi_utils-git", "octopi-git"])
    ));

    let built: Vec<&str> = actions
        .iter()
        .filter_map(|action| match action {
            PlannedAction::User(UserAction::BuildAur { base, run: _ }) => Some(base.as_str()),
            PlannedAction::Root(_) | PlannedAction::User(_) => None,
        })
        .collect();

    assert_eq!(built, ["alpm_octopi_utils-git", "octopi-git"]);
}

#[test]
fn a_refused_route_plans_the_paru_path_and_reads_no_order() {
    let orders = octopi_orders();

    let plan = split_plan(refused_route(), &octopi_copy(&[]), &orders).unwrap();

    assert!(matches!(
        &plan.steps()[0].action,
        PlannedAction::User(UserAction::InstallAur { packages })
            if packages.iter().map(PackageName::as_str).eq(["octopi-git"])
    ));
    assert_eq!(orders.asked(), 0);
}

#[test]
fn a_build_without_its_order_stops_before_any_plan_with_its_reason() {
    let orders = FakeAurOrders::new(vec![("octopi-git", Err("paru is not installed"))]);
    let refused = split_plan(accepted_route(), &octopi_copy(&[]), &orders).unwrap_err();

    assert_eq!(refused.exit(), Exit::Unavailable);
    assert!(
        refused
            .message()
            .starts_with("`octopi-git` has no build order: paru could not be asked"),
        "{}",
        refused.message()
    );
}

#[test]
fn a_route_the_helper_refuses_names_the_build_folder_for_the_missing_order() {
    let plan = plan_install(
        &[name("octopi-git")],
        &octopi_copy(&[]),
        &snapshot_with(accepted_route()),
        &octopi_orders(),
        Err(RouteProblem::NoAccount(1000)),
    );

    assert_eq!(
        plan.unwrap_err().message(),
        "`octopi-git` has no build order: its build folder cannot be used\nuser id 1000 has no account in /etc/passwd"
    );
}

#[test]
fn a_conflict_with_an_installed_package_refuses_the_split_path_and_names_it() {
    let refused = split_plan(
        accepted_route(),
        &octopi_copy(&["paru-bin"]),
        &octopi_orders(),
    );

    assert_eq!(
        refused.unwrap_err(),
        Refused::Rule(
            "octopi-git conflicts with the installed paru-bin. Remove paru-bin first.".to_owned()
        )
    );
}

#[test]
fn under_sudo_an_aur_install_takes_the_old_path_even_on_an_accepted_route() {
    let mut snapshot = snapshot_with(accepted_route());
    crate::root::mark_sudo_route(&mut snapshot);

    let plan = plan_install(
        &[name("octopi-git")],
        &octopi_copy(&[]),
        &snapshot,
        &octopi_orders(),
        Ok(()),
    )
    .unwrap();

    assert!(matches!(
        &plan.steps()[0].action,
        PlannedAction::User(UserAction::InstallAur { packages: _ })
    ));
}
