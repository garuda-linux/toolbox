//! The repositories' packages and, while the AUR is on, the AUR's.

use std::sync::Arc;

use egui::Context;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::AurCatalog;
use toolbox_core::features::packages::list::Catalog;
use toolbox_core::features::search::Needle;
use toolbox_core::features::search::ranked::top;
use toolbox_plan::PackageName;

use super::{CAP, Look, Part, Pick, Row, Section};
use crate::command::Reveal;
use crate::i18n;
use crate::network::Read;
use crate::packages::Packages;
use crate::page::PageId;
use crate::pages::PageContext;
use crate::pages::packages::repository;

/// Kept so the palette finds its rows again once a read lands.
pub type Lists = (Option<Arc<Catalog>>, Option<Arc<AurCatalog>>);

pub fn lists(context: &PageContext<'_>) -> Lists {
    fn known<T>(read: &Read<Arc<T>>) -> Option<Arc<T>> {
        match read {
            Read::Known(list) => Some(Arc::clone(list)),
            Read::Loading | Read::Unavailable | Read::Failed => None,
        }
    }

    let service = context.service::<Packages>();
    let aur = shows_aur(context).then(|| known(service.aur())).flatten();
    (known(service.index()), aur)
}

pub fn same(one: &Lists, other: &Lists) -> bool {
    fn same<T>(one: Option<&Arc<T>>, other: Option<&Arc<T>>) -> bool {
        match (one, other) {
            (Some(one), Some(other)) => Arc::ptr_eq(one, other),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    }

    same(one.0.as_ref(), other.0.as_ref()) && same(one.1.as_ref(), other.1.as_ref())
}

const fn shows_aur(context: &PageContext<'_>) -> bool {
    context.preferences.settings.packages.show_aur
}

pub fn found(
    ctx: &Context,
    context: &PageContext<'_>,
    needle: &Needle,
    query: &str,
) -> [Option<Part>; 2] {
    let (index, aur) = lists(context);
    let installed = context.snapshot.part::<InstalledPackages>();
    let detail = |source: String, name: &PackageName| {
        if installed.is_some_and(|installed| installed.0.contains(name)) {
            format!("{source} · {}", i18n::text(ctx, "packages-installed"))
        } else {
            source
        }
    };

    let repositories = index.and_then(|catalog| {
        let best = top(catalog.ranked(needle), CAP)?.map(|entry| {
            let shown = detail(repository(ctx, &entry.origin), &entry.name);
            row(&entry.name, shown, false)
        });

        let all = Reveal::Packages {
            search: query.trim().to_owned(),
            aur: false,
        };

        Some(Part::capped(best, PageId::Packages, all.into()))
    });

    let from_aur = aur.and_then(|catalog| {
        let entries = catalog.entries();
        let found = catalog
            .ranked(needle)
            .filter_map(|(rank, place)| Some((rank, &entries.get(place)?.name)));

        let best = top(found, CAP)?.map(|name| {
            let shown = detail(i18n::text(ctx, "packages-tag-aur").into_owned(), name);
            row(name, shown, true)
        });

        let all = Reveal::Packages {
            search: query.trim().to_owned(),
            aur: true,
        };

        Some(Part::capped(best, PageId::Packages, all.into()))
    });

    [repositories, from_aur]
}

fn row(name: &PackageName, detail: String, aur: bool) -> Row {
    Row {
        name: name.as_str().to_owned(),
        detail,
        shortcut: None,
        section: if aur { Section::Aur } else { Section::Packages },
        look: Look::Letter {
            package: name.as_str().to_owned(),
        },
        pick: Pick::Run(
            Reveal::Package {
                name: name.clone(),
                aur,
            }
            .into(),
        ),
    }
}
