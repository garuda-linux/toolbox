//! The left column of the detail page: the facts of the package. Its
//! description is the subtitle under the name. Every value is the
//! database's own text, shown plain. Only a web address opens, and only an
//! `http` or `https` one.

use egui::{Context, Ui};
use toolbox_cli::output::size;
use toolbox_core::features::packages::detail::web_address;
use toolbox_core::features::packages::{InstallReason, PackageDetails};
use toolbox_gui_kit::widgets::{PropertyTexts, property_list, text_link};
use toolbox_log::calendar::Utc;

use crate::i18n;
use crate::pages::packages::row::repository;

pub struct Fact {
    pub label: String,
    pub value: Value,
}

pub enum Value {
    Plain(String),
    /// A version, which someone may paste into a bug report.
    Copied(String),
    Web(String, String),
}

pub fn show(ui: &mut Ui, info: &PackageDetails) {
    let facts = facts(ui.ctx(), info);
    grid(ui, &facts);
}

pub fn grid(ui: &mut Ui, facts: &[Fact]) {
    let ctx = ui.ctx().clone();
    let copy =
        |label: &str| i18n::text_with(&ctx, "packages-copy", &[("label", label)]).into_owned();

    let copied = i18n::text(&ctx, "packages-copied");

    let texts = PropertyTexts {
        copy: &copy,
        copied: &copied,
    };

    property_list(ui, texts, |list| {
        for fact in facts {
            match &fact.value {
                Value::Plain(value) => list.row(&fact.label, value),
                Value::Copied(value) => list.row_copied(&fact.label, value),
                Value::Web(url, shown) => list.row_with(&fact.label, Some(url), move |cell| {
                    let text = format!("{shown} ↗");
                    if text_link(cell, &text, url).on_hover_text(url).clicked() {
                        crate::links::open(cell.ctx(), url);
                    }
                }),
            }
        }
    });
}

/// The rows of the grid. Lists that are empty and the installed rows of a
/// package that is not installed are left out.
fn facts(ctx: &Context, info: &PackageDetails) -> Vec<Fact> {
    let fact = |id: &str, value: Value| Fact {
        label: i18n::text(ctx, id).into_owned(),
        value,
    };

    let mut facts = vec![fact(
        "packages-info-repository",
        Value::Plain(repository(ctx, &info.origin)),
    )];

    if let Some(own) = info
        .installed
        .as_ref()
        .filter(|own| own.version != info.version)
    {
        facts.push(fact(
            "packages-info-installed-version",
            Value::Copied(own.version.clone()),
        ));
    }

    facts.push(fact("packages-info-size", Value::Plain(size(info.size))));

    let listed = |id: &str, values: &[String]| {
        (!values.is_empty()).then(|| fact(id, Value::Plain(values.join(", "))))
    };

    facts.extend(listed("packages-info-licenses", &info.licenses));
    facts.extend(listed("packages-info-groups", &info.groups));

    facts.extend(installed(ctx, info));

    if let Some(url) = &info.url {
        facts.push(fact("packages-info-url", web(url)));
    }

    facts.extend(listed("packages-info-provides", &info.provides));
    facts.extend(listed("packages-info-conflicts", &info.conflicts));
    facts.extend(listed("packages-info-replaces", &info.replaces));
    facts
}

/// A web address the database names opens only when it is an `http` or
/// `https` one, else it shows as plain text.
pub fn web(url: &str) -> Value {
    web_address(url).map_or_else(
        || Value::Plain(url.to_owned()),
        |shown| Value::Web(url.to_owned(), shown),
    )
}

pub fn installed(ctx: &Context, info: &PackageDetails) -> Vec<Fact> {
    let Some(own) = &info.installed else {
        return Vec::new();
    };

    let fact = |id: &str, value: String| Fact {
        label: i18n::text(ctx, id).into_owned(),
        value: Value::Plain(value),
    };

    let reason = match own.reason {
        InstallReason::Explicit => "packages-reason-explicit",
        InstallReason::Dependency => "packages-reason-dependency",
    };

    own.date
        .map(|date| fact("packages-info-date", day(date)))
        .into_iter()
        .chain([fact(
            "packages-info-reason",
            i18n::text(ctx, reason).into_owned(),
        )])
        .collect()
}

/// The day of `seconds` since the epoch, as `2025-01-10`, in UTC.
pub fn day(seconds: i64) -> String {
    let date = Utc::from_unix(u64::try_from(seconds).unwrap_or_default());
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}
