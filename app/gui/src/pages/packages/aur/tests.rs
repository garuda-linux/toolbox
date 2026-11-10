use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_aur::AurSource as _;
use toolbox_core::Toggle;
use toolbox_core::features::packages::aur::AurCatalog;
use toolbox_core::features::packages::list::Filter;
use toolbox_gui_kit::Loadable;
use toolbox_plan::PackageName;
use toolbox_system::{PendingUpdate, ProbeError, Status};

use super::super::PackagesPage;
use super::super::tests::{catalog, fixture, listed, name};
use crate::command::{Command, PackagesCommand, UiCommand};
use crate::packages::{Packages, Read, SampleAur};
use crate::page::PageId;
use crate::testing::PageFixture;

fn aur() -> Arc<AurCatalog> {
    Arc::new(AurCatalog::new(SampleAur.index().unwrap()))
}

fn on(
    page: PackagesPage,
) -> egui_kittest::Harness<'static, crate::testing::PageState<PackagesPage>> {
    let mut fixture = fixture(None);
    fixture.settings.packages.show_aur = true;
    fixture.services.packages =
        Packages::showing(Read::Known(catalog()), None).with_aur(Read::Known(aur()));

    let mut harness = fixture.harness(page);
    harness.run();
    harness
}

fn aur_page() -> PackagesPage {
    PackagesPage {
        filter: Filter::Aur,
        aur_asked: true,
        ..PackagesPage::default()
    }
}

#[test]
fn while_the_aur_is_off_its_chip_hides_and_a_hint_leads_to_settings() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();

    assert!(harness.query_by_label("AUR").is_none());
    harness
        .get_by_role_and_label(Role::Link, "Turn them on in Settings")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(UiCommand::Navigate(PageId::Settings))]
    );
}

#[test]
fn the_aur_chip_asks_for_the_copy_only_once_it_shows() {
    let mut fixture = fixture(None);
    fixture.settings.packages.show_aur = true;
    let mut harness = fixture.harness(PackagesPage::default());
    harness.run();
    assert!(
        harness.state().fixture.sent().is_empty(),
        "asked before the chip"
    );

    harness.get_by_role_and_label(Role::Button, "AUR").click();
    harness.run();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::OpenAur)]
    );
}

#[test]
fn the_aur_chip_lists_the_most_popular_first_with_votes_and_marks() {
    let harness = on(aur_page());

    for package in ["paru-bin", "yay-bin", "toolbox-sample-git"] {
        assert!(listed(&harness, package), "{package}");
    }

    assert!(!listed(&harness, "gimp"));
    assert!(harness.query_by_label("Out of date").is_some());
    assert!(harness.query_all_by_label("1.0-1 · 42 votes").count() >= 3);

    let tops: Vec<f32> = ["paru-bin", "yay-bin", "toolbox-sample-git"]
        .iter()
        .map(|package| {
            harness
                .get_by_role_and_label(Role::ListItem, package)
                .rect()
                .top()
        })
        .collect();

    assert!(tops.is_sorted(), "{tops:?}");
}

#[test]
fn install_asks_about_the_build_script_before_it_queues() {
    let mut harness = on(aur_page());

    harness.get_by_label("Install yay-bin").click();
    harness.run();
    assert!(harness.state().fixture.sent().is_empty(), "queued at once");
    assert!(
        harness
            .query_by_label_contains("paru shows the build script in the Console")
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::Button, "Install")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::ToggleAur {
            package: name("yay-bin"),
            wish: Toggle::On,
        })]
    );
}

#[test]
fn an_aur_row_opens_the_aur_details() {
    let mut harness = on(aur_page());

    harness
        .get_by_role_and_label(Role::ListItem, "paru-bin")
        .click();
    harness.run();
    harness.run();

    assert!(harness.state().page.open_aur);
    for label in [
        "Maintainer",
        "Votes",
        "Popularity",
        "First submitted",
        "Last updated",
        "Make dependencies",
        "https://aur.archlinux.org/packages/paru-bin",
    ] {
        assert!(harness.query_by_label(label).is_some(), "{label}");
    }
}

#[test]
fn the_updates_chip_counts_the_aur_updates_home_counts() {
    let mut fixture = fixture(None);
    let update = |text: &str| PendingUpdate {
        name: PackageName::new(text).unwrap(),
        from: "1".to_owned(),
        to: "2".to_owned(),
    };

    fixture.status = Some(Loadable::Loaded(Arc::new(Status {
        updates: Ok(vec![update("gimp")]),
        aur_updates: Ok(vec![update("paru")]),
        merges: Ok(Vec::new()),
        last_update: Err(ProbeError::Unavailable),
        health: Ok(Vec::new()),
    })));

    let page = PackagesPage {
        filter: Filter::Updates,
        ..PackagesPage::default()
    };

    let mut harness = fixture.harness(page);
    harness.run();

    assert!(listed(&harness, "gimp"));
    assert!(listed(&harness, "paru"));
}

/// Without pacman's databases (the self-test's Nix runner) the AUR still
/// shows under the notice.
#[test]
fn the_aur_chip_works_without_pacmans_databases() {
    let mut fixture = PageFixture::default();
    fixture.settings.packages.show_aur = true;
    fixture.services.packages =
        Packages::showing(Read::Unavailable, None).with_aur(Read::Known(aur()));

    let mut harness = fixture.harness(aur_page());
    harness.run();

    assert!(
        harness
            .query_by_label_contains("package databases of pacman could not be read")
            .is_some()
    );
    assert!(listed(&harness, "paru-bin"));
}

#[test]
fn without_pacmans_databases_the_other_chips_show_only_the_notice() {
    let mut fixture = PageFixture::default();
    fixture.settings.packages.show_aur = true;
    fixture.services.packages =
        Packages::showing(Read::Unavailable, None).with_aur(Read::Known(aur()));

    let mut harness = fixture.harness(PackagesPage::default());
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "AUR 3")
            .is_some()
    );
    assert!(harness.query_by_label("No packages to show").is_none());
}
