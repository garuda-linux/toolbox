use std::collections::BTreeSet;

use egui::{Key, Label, Ui};
use toolbox_core::features::configs::{Row, RowKind, rows};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowHeights, RowList, RowState, SectionHeading, empty_state, pill_button,
    rich, row_height, split_row, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::texts;
use crate::command::ConfigsCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::more::{item, more};

const CONTROLS_WIDTH: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.row_actions_narrow;

struct Item {
    row: Row,
    name: String,
}

struct Section {
    title: String,
    icon: String,
    items: Vec<Item>,
}

impl ListItem for Item {
    fn key(&self) -> &str {
        &self.row.key
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    search: &mut String,
    expanded: &mut BTreeSet<String>,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let configs = context.configs;
    let catalog = match configs.catalog() {
        Ok(catalog) => catalog,
        Err(problem) => {
            let (title, detail) = texts::unreadable(&ctx, problem);
            empty_state(ui, icons::HELP_ABOUT, &title, &detail);
            return;
        }
    };

    let Some(listing) = configs.listing() else {
        empty_state(ui, icons::DOCUMENT_PREVIEW, &text("configs-reading"), "");
        return;
    };

    let sections = sections(&ctx, rows(catalog, listing, expanded, search));
    if sections.is_empty() {
        nothing(ui, search);
        return;
    }

    for section in &sections {
        let icon = section_icon(&section.icon);
        SectionHeading::new(icon, &section.title, "").show(ui);
        ui.add_space(ui.tokens().sizes.heading_gap);
        let name = i18n::text_with(&ctx, "configs-list", &[("section", &section.title)]);
        let heights = RowHeights::Measured {
            estimate: row_height(ui, text_block_height(ui, 2)),
        };

        RowList::new(("configs", &section.icon, &section.title), &name, heights).show(
            ui,
            &section.items,
            |ui, item, state| row(ui, context, (item, state), expanded),
        );

        ui.add_space(ui.tokens().sizes.section_gap);
    }
}

fn nothing(ui: &mut Ui, search: &mut String) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    if search.is_empty() {
        empty_state(ui, icons::DOCUMENT_PREVIEW, &text("configs-none"), "");
        return;
    }

    empty_state(
        ui,
        icons::SYSTEM_SEARCH,
        &text("configs-no-match"),
        &text("configs-search-detail"),
    );
    if pill_button(ui, &text("configs-clear-search"), ButtonKind::Soft).clicked() {
        search.clear();
    }
}

fn sections(ctx: &egui::Context, rows: Vec<Row>) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    for row in rows {
        if row.kind == RowKind::Section {
            sections.push(Section {
                title: catalog_text(ctx, &row.key, &row.name),
                icon: row.icon,
                items: Vec::new(),
            });
            continue;
        }

        let name = if row.kind == RowKind::Child {
            row.name.clone()
        } else {
            catalog_text(ctx, &row.key, &row.name)
        };

        if let Some(section) = sections.last_mut() {
            section.items.push(Item { row, name });
        }
    }

    sections
}

fn row(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (item, state): (&Item, &RowState),
    expanded: &mut BTreeSet<String>,
) {
    let row = &item.row;
    let activated = state.toggle
        || (state.response.has_focus() && ui.input(|input| input.key_pressed(Key::Enter)));

    let hint = hover(ui.ctx(), row);
    if !hint.is_empty() {
        state.response.clone().on_hover_text(hint);
    }

    split_row(
        ui,
        CONTROLS_WIDTH,
        |ui| words(ui, item),
        |ui| controls(ui, context, (row, &item.name), activated, expanded),
    );
}

fn words(ui: &mut Ui, item: &Item) {
    let ctx = ui.ctx().clone();
    let tokens = ui.tokens();
    let row = &item.row;
    toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
        let title = rich(ui, &item.name, Weight::SemiBold, BODY, tokens.text);
        ui.add(Label::new(title).truncate());
        if row.kind == RowKind::Folder {
            let count =
                i18n::text_with(&ctx, "configs-files", &[("count", &row.count.to_string())]);

            // A count is no state, so it takes no dot.
            let count = rich(ui, &count, Weight::Medium, SMALL, tokens.secondary);
            ui.add(Label::new(count).selectable(false));
        }
    });

    ui.add(
        Label::new(rich(
            ui,
            &row.path,
            Weight::Regular,
            SMALL,
            tokens.secondary,
        ))
        .truncate(),
    );

    // The reason a file is not edited is the one line worth reading in
    // full, so it wraps where the path is cut.
    let status = texts::row_status(&ctx, row);
    if !status.is_empty() {
        ui.add(Label::new(rich(ui, &status, Weight::Regular, SMALL, tokens.secondary)).wrap());
    }
}

/// Return or Space on the focused row.
fn controls(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (row, name): (&Row, &str),
    activated: bool,
    expanded: &mut BTreeSet<String>,
) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let clicked = if row.kind == RowKind::Folder {
        let id = if row.expanded {
            "configs-hide-files"
        } else {
            "configs-show-files"
        };

        pill_button(ui, &text(id), ButtonKind::Soft).clicked()
    } else {
        action(ui, context, row)
    };

    // A file of the user may also open in the desktop's app for it, for
    // whoever prefers a graphical editor.
    let in_app = !row.admin && row.kind != RowKind::Folder;
    if in_app || !row.url.is_empty() {
        more(ui, name, |ui| {
            if in_app && item(ui, "configs-open-in-app", true) {
                context.commands.send(ConfigsCommand::Open(row.key.clone()));
            }

            if !row.url.is_empty() && item(ui, "configs-wiki-open", true) {
                crate::links::open(&ctx, &row.url);
            }
        });
    }

    if !(clicked || activated) {
        return;
    }

    let key = row.key.clone();
    let command = match (row.kind, row.action) {
        (RowKind::Folder, _) => {
            if !expanded.remove(&key) {
                expanded.insert(key);
            }

            return;
        }
        (_, "edit") if editable(context) => ConfigsCommand::Edit(key),
        (_, "view") => ConfigsCommand::View(key),
        _ => return,
    };

    context.commands.send(command);
}

fn action(ui: &mut Ui, context: &PageContext<'_>, row: &Row) -> bool {
    let (id, enabled) = match row.action {
        "edit" => ("configs-edit", editable(context)),
        "view" => ("configs-view", true),
        _ => return false,
    };

    let label = i18n::text(ui.ctx(), id);
    ui.add_enabled_ui(enabled, |ui| pill_button(ui, &label, ButtonKind::Soft))
        .inner
        .clicked()
}

/// One file is edited at a time, and not while another plan runs.
const fn editable(context: &PageContext<'_>) -> bool {
    context.configs.session().phase().is_free() && !context.busy
}

fn hover(ctx: &egui::Context, row: &Row) -> String {
    let id = row.key.split('/').next().unwrap_or(&row.key);
    let description = if row.description.is_empty() {
        String::new()
    } else {
        i18n::text_or(
            ctx,
            &format!("catalog-config-files-{id}-description"),
            &row.description,
        )
        .into_owned()
    };

    let link = row
        .link
        .then(|| texts::link(ctx, row))
        .filter(|link| *link != texts::row_status(ctx, row));

    link.into_iter()
        .chain((!description.is_empty()).then_some(description))
        .collect::<Vec<_>>()
        .join("\n")
}

fn catalog_text(ctx: &egui::Context, id: &str, english: &str) -> String {
    i18n::text_or(ctx, &format!("catalog-config-files-{id}"), english).into_owned()
}

/// The catalog's icon of a section, or a plain text file for a name the
/// app does not bundle.
pub(super) fn section_icon(name: &str) -> icons::Icon {
    icons::named(name).unwrap_or(icons::TEXT_X_GENERIC)
}
