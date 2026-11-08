//! The columns of an AUR package's detail page. Every value is the AUR's
//! own text, cleaned when the dump was read and shown plain. Only the
//! website and the AUR page open, the website only when it is an `http`
//! or `https` address.

use egui::{Context, Ui};
use toolbox_aur::package_page;
use toolbox_cli::output::size;
use toolbox_core::features::packages::aur::AurPackage;
use toolbox_core::features::packages::{OptionalDependency, PackageDetails};
use toolbox_plan::PackageName;

use super::super::super::Unfolded;
use super::super::facts::{Fact, Value, day, grid, installed, web};
use super::super::relations::{gap, names, optional};
use crate::i18n;

const AUR_HOST: &str = "aur.archlinux.org";

pub fn show(ui: &mut Ui, entry: &AurPackage, local: Option<&PackageDetails>) {
    let facts = facts(ui.ctx(), entry, local);
    grid(ui, &facts);
}

fn facts(ctx: &Context, entry: &AurPackage, local: Option<&PackageDetails>) -> Vec<Fact> {
    let fact = |id: &str, value: Value| Fact {
        label: i18n::text(ctx, id).into_owned(),
        value,
    };

    let plain = |id: &str, value: String| fact(id, Value::Plain(value));

    let maintainer = entry
        .maintainer
        .clone()
        .unwrap_or_else(|| i18n::text(ctx, "packages-aur-orphaned").into_owned());

    let mut facts = vec![
        plain("packages-aur-maintainer", maintainer),
        plain("packages-aur-votes-label", entry.votes.to_string()),
        plain(
            "packages-aur-popularity",
            format!("{:.2}", entry.popularity),
        ),
        plain("packages-aur-submitted", day(entry.first_submitted)),
        plain("packages-aur-updated", day(entry.last_modified)),
    ];

    if let Some(flagged) = entry.out_of_date {
        facts.push(plain("packages-aur-out-of-date-since", day(flagged)));
    }

    if !entry.licenses.is_empty() {
        facts.push(plain("packages-info-licenses", entry.licenses.join(", ")));
    }

    if let Some(url) = &entry.url {
        facts.push(fact("packages-info-url", web(url)));
    }

    // The host alone: the whole address would push a long name over the
    // column's edge. The link's name and tooltip hold all of it.
    let page = package_page(&entry.name);
    facts.push(fact(
        "packages-aur-page",
        Value::Web(page, AUR_HOST.to_owned()),
    ));

    if let Some(info) = local {
        if let Some(own) = info
            .installed
            .as_ref()
            .filter(|own| own.version != entry.version)
        {
            facts.push(fact(
                "packages-info-installed-version",
                Value::Copied(own.version.clone()),
            ));
        }

        facts.push(plain("packages-info-size", size(info.size)));
        facts.extend(installed(ctx, info));
    }

    facts
}

/// Returns the package whose name was clicked.
pub fn relations(
    ui: &mut Ui,
    unfolded: &mut Unfolded,
    entry: &AurPackage,
    lookup: &dyn Fn(&str) -> Option<PackageName>,
) -> Option<PackageName> {
    let space = gap(ui);
    let depends = names(
        ui,
        "packages-depends",
        &entry.depends,
        lookup,
        &mut unfolded.depends,
    );

    ui.add_space(space);
    let make = names(
        ui,
        "packages-aur-make-depends",
        &entry.make_depends,
        lookup,
        &mut unfolded.make_depends,
    );

    ui.add_space(space);
    let wanted: Vec<OptionalDependency> =
        entry.opt_depends.iter().map(|line| split(line)).collect();

    let optional = optional(ui, &wanted, lookup, &mut unfolded.optional);

    // Provides and conflicts name what the package stands for, never a
    // package to open.
    let nothing = |_: &str| None;
    ui.add_space(space);
    let mut provides_all = true;
    _ = names(
        ui,
        "packages-info-provides",
        &entry.provides,
        &nothing,
        &mut provides_all,
    );

    ui.add_space(space);
    let mut conflicts_all = true;
    _ = names(
        ui,
        "packages-info-conflicts",
        &entry.conflicts,
        &nothing,
        &mut conflicts_all,
    );

    depends.or(make).or(optional)
}

/// `name: what it adds`, as makepkg writes an optional dependency.
fn split(line: &str) -> OptionalDependency {
    let (name, reason) = line.split_once(':').unwrap_or((line, ""));
    OptionalDependency {
        name: name.trim().to_owned(),
        reason: reason.trim().to_owned(),
    }
}
