use std::collections::BTreeSet;

use toolbox_aur::{AurError, AurSource};
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::{AurIndex, AurPackage};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{PackageName, Plan, PlannedAction, UserAction};

use super::{aur_info, aur_info_text, aur_listing, found, install_onto, waiting_text};
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
        })
    }
}

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn package(text: &str, popularity: f64) -> AurPackage {
    AurPackage {
        name: name(text),
        version: "1.0-1".to_owned(),
        description: format!("About {text}"),
        url: Some("https://example.org".to_owned()),
        votes: 7,
        popularity,
        out_of_date: None,
        maintainer: Some("someone".to_owned()),
        first_submitted: 1_736_467_200,
        last_modified: 1_736_467_200,
        depends: vec!["glibc".to_owned()],
        make_depends: vec!["cargo".to_owned()],
        opt_depends: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        licenses: vec!["MIT".to_owned()],
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
    snapshot
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
    let text = aur_info_text(&info);
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
            &Aur(Some(AurError::Download("offline".to_owned()))),
            &snapshot()
        ),
        Err(Refused::Unavailable(_))
    ));
}

#[test]
fn a_waiting_copy_names_the_day_of_the_next_download() {
    assert!(waiting_text(1_736_467_200).contains("2025-01-10"));
}
