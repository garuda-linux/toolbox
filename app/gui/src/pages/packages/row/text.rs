//! The text of a package row: its title with the tags and state, the small
//! lines under it, and the tags that say what kind of package it is.

use egui::{Context, Label, Ui};
use toolbox_cli::output::size;
use toolbox_core::features::packages::{IndexEntry, InstallReason, Origin};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{Tone, rich, status_dot, tag_on_row, title_line};
use toolbox_plan::PackageName;

use super::PackageRow;
use crate::i18n;
use crate::i18n::Arg;

/// A selected row's text takes the selection's own colour, as Darkly draws a
/// selected item. The full-width row adds its origin line under the
/// description.
pub(super) fn text<R>(
    ui: &mut Ui,
    row: &R,
    word: Option<(&'static str, Tone)>,
    selected: bool,
    origin: bool,
) where
    R: PackageRow,
{
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let (name, secondary) = if selected {
        (tokens.on_tint, tokens.on_tint)
    } else {
        (tokens.text, tokens.secondary)
    };

    title_line(ui, false, |ui| {
        let title = rich(ui, row.package().as_str(), Weight::SemiBold, BODY, name);
        ui.add(Label::new(title).truncate());

        row.marks(ui, selected);
        if let Some((id, tone)) = word {
            status_dot(ui, &i18n::text(&ctx, id), tone);
        }
    });

    // The small lines stack as one block, as the row's height counts them.
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let detail = row.detail_line(&ctx);
        let lines: &[&str] = if origin {
            &[row.description(), detail.as_str()]
        } else {
            &[row.description()]
        };

        for line in lines {
            let small = rich(ui, line, Weight::Regular, SMALL, secondary);
            ui.add(Label::new(small).truncate());
        }
    });
}

/// "extra · 3.0-1 · 12.3 MiB", the installed version for an installed
/// package.
pub fn origin_line(ctx: &Context, entry: &IndexEntry) -> String {
    let version = entry
        .installed
        .as_ref()
        .map_or(entry.version.as_str(), |installed| {
            installed.version.as_str()
        });

    [
        repository(ctx, &entry.origin),
        version.to_owned(),
        size(entry.size),
    ]
    .join(" · ")
}

/// The tags beside the name: pulled in as a dependency, not from a
/// repository.
pub fn tags(entry: &IndexEntry) -> impl Iterator<Item = &'static str> {
    let dependency = entry
        .installed
        .as_ref()
        .is_some_and(|installed| installed.reason == InstallReason::Dependency);

    let foreign = entry.origin == Origin::Foreign;

    [
        dependency.then_some("packages-tag-dependency"),
        foreign.then_some("packages-tag-foreign"),
    ]
    .into_iter()
    .flatten()
}

pub fn repository(ctx: &Context, origin: &Origin) -> String {
    match origin {
        Origin::Repository(repo) => repo.clone(),
        Origin::Foreign => i18n::text(ctx, "packages-no-repository").into_owned(),
    }
}

pub(super) fn optional_tag(ui: &mut Ui, by: &[PackageName], selected: bool) {
    let Some((first, others)) = by.split_first() else {
        return;
    };

    let ctx = ui.ctx().clone();
    let id = if others.is_empty() {
        "packages-tag-optional-for"
    } else {
        "packages-tag-optional-for-more"
    };

    let args = [
        ("first", Arg::from(first.as_str())),
        ("more", Arg::from(others.len())),
    ];

    let response = tag_on_row(ui, &i18n::text_with(&ctx, id, &args), selected);

    if !others.is_empty() {
        let names = by
            .iter()
            .map(PackageName::as_str)
            .collect::<Vec<_>>()
            .join(", ");

        let all = i18n::text_with(
            &ctx,
            "packages-tag-optional-for-all",
            &[("names", Arg::from(&names))],
        );

        _ = response.on_hover_text(all);
    }
}
