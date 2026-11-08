mod facts;

use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::packages::Origin;
use toolbox_core::features::packages::aur::{AurCatalog, AurPackage};
use toolbox_core::features::packages::detail::target;
use toolbox_core::features::packages::graph::dependency_name;
use toolbox_core::features::packages::list::Catalog;
use toolbox_gui_kit::UiExt as _;
use toolbox_plan::PackageName;

use super::super::PackagesPage;
use super::super::aur::row::AurRow;
use super::{below, catalog, gone, header_of, loading, rule, split};
use crate::pages::PageContext;

pub fn header(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    package: &PackageName,
) {
    let aur = context.packages.aur().known().map(Arc::clone);
    let entry = aur.as_ref().and_then(|aur| aur.entry(package));
    let repo = catalog(context).unwrap_or_default();
    let shown = entry.map(|entry| AurRow { entry, repo: &repo });
    header_of(page, ui, context, package, shown);
}

pub fn body(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    package: &PackageName,
) {
    let Some(aur) = context.packages.aur().known().map(Arc::clone) else {
        loading(ui);
        return;
    };

    let Some(entry) = aur.entry(package) else {
        gone(ui, package);
        return;
    };

    let repo = catalog(context);
    let details = context
        .packages
        .details()
        .filter(|details| details.package == *package);

    let local = details
        .and_then(|details| details.info.known())
        .and_then(Option::as_ref)
        .filter(|info| info.installed.is_some());

    let lookup = |value: &str| lead(repo.as_deref(), &aur, value);
    let unfolded = &mut page.unfolded;
    let opened = split(
        ui,
        |ui| facts::show(ui, entry, local),
        |ui| facts::relations(ui, unfolded, entry, &lookup),
    );

    if let Some(next) = opened {
        let to_aur = is_aur(repo.as_deref(), &aur, &next);
        page.follow(&next, to_aur, context);
        return;
    }

    if let (Some(info), Some(details)) = (local, details) {
        rule(ui);
        below::files(page, ui, context, info);
        ui.add_space(ui.tokens().sizes.section_gap);
        below::history(page, ui, context, &details.history);
    }
}

/// The package a dependency of an AUR package leads to: a repository's
/// or an installed one first, as pacman would pick it, else the AUR's.
fn lead(repo: Option<&Catalog>, aur: &AurCatalog, dependency: &str) -> Option<PackageName> {
    repo.and_then(|repo| target(repo, dependency)).or_else(|| {
        let named = PackageName::new(dependency_name(dependency)).ok()?;
        aur.entry(&named)
            .map(|entry: &AurPackage| entry.name.clone())
    })
}

/// Whether `package` opens on its AUR side: no repository offers it and
/// the AUR has it.
fn is_aur(repo: Option<&Catalog>, aur: &AurCatalog, package: &PackageName) -> bool {
    let offered = repo
        .and_then(|repo| repo.entry(package))
        .is_some_and(|entry| matches!(entry.origin, Origin::Repository(_)));

    !offered && aur.entry(package).is_some()
}
