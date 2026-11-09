use toolbox_state::aur::{AurIndex, AurPackage};
use toolbox_state::packages::PackageGraph;

use super::{AurCatalog, row_state, toggle};
use crate::domain::Queued;
use crate::domains::{AurPackages, InstalledPackages, Packages};
use crate::features::packages::list::RowState;
use crate::features::packages::queue::Refusal;
use crate::testing::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

fn aur(name: &str, description: &str, popularity: f64) -> AurPackage {
    AurPackage {
        name: package(name),
        version: "1.0-1".to_owned(),
        description: description.to_owned(),
        url: None,
        votes: 1,
        popularity,
        out_of_date: None,
        maintainer: None,
        first_submitted: 0,
        last_modified: 0,
        depends: Vec::new(),
        make_depends: Vec::new(),
        opt_depends: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        licenses: Vec::new(),
        keywords: Vec::new(),
    }
}

fn catalog() -> AurCatalog {
    let mut tagged = aur("vscodium-bin", "Free build of the editor", 9.0);
    tagged.keywords = vec!["IDE".to_owned()];

    AurCatalog::new(AurIndex {
        packages: vec![
            aur("yay-bin", "Yet another yogurt, prebuilt", 4.0),
            aur("paru", "Feature packed AUR helper", 20.0),
            aur("paru-bin", "Feature packed AUR helper, prebuilt", 30.0),
            aur("aurutils", "Helper tools for paru users", 50.0),
            tagged,
        ],
        downloaded: 1_700_000_000,
    })
}

fn names(catalog: &AurCatalog, places: &[usize]) -> Vec<String> {
    places
        .iter()
        .map(|&place| catalog.entries()[place].name.to_string())
        .collect()
}

#[test]
fn without_a_search_the_most_popular_package_comes_first() {
    let catalog = catalog();

    assert_eq!(
        names(&catalog, &catalog.view("")),
        ["aurutils", "paru-bin", "paru", "vscodium-bin", "yay-bin"]
    );
}

#[test]
fn the_exact_name_comes_first_then_names_that_start_with_the_search() {
    let catalog = catalog();

    assert_eq!(
        names(&catalog, &catalog.view("PARU")),
        ["paru", "paru-bin", "aurutils"]
    );
}

#[test]
fn the_search_finds_keywords_and_descriptions() {
    let catalog = catalog();

    assert_eq!(names(&catalog, &catalog.view("ide")), ["vscodium-bin"]);
    assert_eq!(names(&catalog, &catalog.view("yogurt")), ["yay-bin"]);
    assert_eq!(catalog.view("nothing like it"), Vec::<usize>::new());
}

#[test]
fn a_package_is_found_by_name() {
    let catalog = catalog();

    assert_eq!(
        catalog
            .entry(&package("paru"))
            .map(|entry| entry.popularity),
        Some(20.0_f64)
    );
    assert_eq!(catalog.entry(&package("pacman")), None);
    assert_eq!(catalog.downloaded(), 1_700_000_000);
}

fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(names)));
    snapshot.insert(PackageGraph::default());
    snapshot
}

#[test]
fn install_queues_an_aur_build_and_a_second_click_takes_it_back() {
    let snapshot = installed(&["paru"]);
    let mut changes = ChangeSet::default();
    let yay = package("yay-bin");

    toggle(&yay, Toggle::On, &snapshot, &mut changes).unwrap();
    assert_eq!(changes.wish::<AurPackages>(&yay), Some(&Queued));
    assert_eq!(
        row_state(&yay, &snapshot, &changes),
        RowState {
            installed: false,
            queued: Some(Toggle::On),
        }
    );

    toggle(&yay, Toggle::Off, &snapshot, &mut changes).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn remove_of_an_installed_aur_package_is_a_package_removal() {
    let snapshot = installed(&["paru"]);
    let mut changes = ChangeSet::default();
    let paru = package("paru");

    toggle(&paru, Toggle::Off, &snapshot, &mut changes).unwrap();
    assert_eq!(changes.wish::<Packages>(&paru), Some(&Toggle::Off));
    assert_eq!(
        row_state(&paru, &snapshot, &changes).queued,
        Some(Toggle::Off)
    );

    toggle(&paru, Toggle::On, &snapshot, &mut changes).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn nothing_is_queued_before_the_installed_packages_are_known() {
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(
            &package("yay-bin"),
            Toggle::On,
            &Snapshot::default(),
            &mut changes
        ),
        Err(Refusal::Unread)
    );
}
