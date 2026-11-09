//! The right column of the detail page: what the package depends on,
//! what requires it and what it can use, each with its count. A name a
//! known package answers to opens that package. Long lists show their
//! first names and "+N more".

use egui::{Label, Ui};
use toolbox_core::features::packages::detail::{NAMES_SHOWN, shown, target};
use toolbox_core::features::packages::list::Catalog;
use toolbox_core::features::packages::{OptionalDependency, PackageDetails};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{minor_heading, rich, text_button, text_link};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_plan::PackageName;

use super::super::{Unfolded, text_line};
use crate::i18n;

/// Returns the package whose name was clicked.
pub fn show(
    ui: &mut Ui,
    unfolded: &mut Unfolded,
    catalog: Option<&Catalog>,
    info: &PackageDetails,
) -> Option<PackageName> {
    let mut opened = None;
    let gap = gap(ui);
    let lookup = |value: &str| target(catalog?, value);

    let depends = names(
        ui,
        "packages-depends",
        &info.depends,
        &lookup,
        &mut unfolded.depends,
    );

    ui.add_space(gap);

    let required = names(
        ui,
        "packages-required-by",
        &info.required_by,
        &lookup,
        &mut unfolded.required_by,
    );

    ui.add_space(gap);
    let optional = optional(ui, &info.optional, &lookup, &mut unfolded.optional);

    for clicked in [depends, required, optional] {
        if opened.is_none() {
            opened = clicked;
        }
    }

    opened
}

pub fn gap(ui: &Ui) -> f32 {
    ui.tokens().sizes.section_gap / 2.0
}

/// `lookup` answers is a link to that package.
pub fn names(
    ui: &mut Ui,
    id: &str,
    values: &[String],
    lookup: &dyn Fn(&str) -> Option<PackageName>,
    all: &mut bool,
) -> Option<PackageName> {
    heading(ui, id, values.len());
    if values.is_empty() {
        none(ui);
        return None;
    }

    let part = shown(values, NAMES_SHOWN, *all);
    let mut opened = None;
    text_line(ui, |ui| {
        ui.spacing_mut().item_spacing.x = ui.tokens().sizes.card_gap;
        for value in part.items {
            let clicked = name(ui, value, lookup);
            if opened.is_none() {
                opened = clicked;
            }
        }

        more(ui, part.hidden, all);
    });

    opened
}

pub fn optional(
    ui: &mut Ui,
    values: &[OptionalDependency],
    lookup: &dyn Fn(&str) -> Option<PackageName>,
    all: &mut bool,
) -> Option<PackageName> {
    heading(ui, "packages-optional", values.len());
    if values.is_empty() {
        none(ui);
        return None;
    }

    let tokens = tokens::current(ui.ctx());
    let part = shown(values, NAMES_SHOWN, *all);
    let mut opened = None;
    for value in part.items {
        text_line(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let clicked = name(ui, value.name.as_str(), lookup);
            if opened.is_none() {
                opened = clicked;
            }

            if !value.reason.is_empty() {
                let reason = format!(": {}", value.reason);
                ui.add(
                    Label::new(rich(ui, &reason, Weight::Regular, BODY, tokens.secondary)).wrap(),
                );
            }
        });
    }

    more(ui, part.hidden, all);
    opened
}

fn name(
    ui: &mut Ui,
    value: &str,
    lookup: &dyn Fn(&str) -> Option<PackageName>,
) -> Option<PackageName> {
    let Some(package) = lookup(value) else {
        let tokens = tokens::current(ui.ctx());
        ui.label(rich(ui, value, Weight::Regular, BODY, tokens.text));
        return None;
    };

    text_link(ui, value, value).clicked().then_some(package)
}

fn more(ui: &mut Ui, hidden: usize, all: &mut bool) {
    if hidden == 0 {
        return;
    }

    let ctx = ui.ctx().clone();
    let count = hidden.to_string();
    let text = i18n::text_with(&ctx, "packages-more", &[("count", &count)]);
    let name = i18n::text_with(&ctx, "packages-more-name", &[("count", &count)]);
    if text_button(ui, &text, &name).clicked() {
        *all = true;
    }
}

fn heading(ui: &mut Ui, id: &str, count: usize) {
    let title = i18n::text(ui.ctx(), id);
    minor_heading(ui, &title, &count.to_string());
}

fn none(ui: &mut Ui) {
    let tokens = tokens::current(ui.ctx());
    let text = i18n::text(ui.ctx(), "packages-value-none");
    ui.label(rich(ui, &text, Weight::Regular, BODY, tokens.secondary));
}
