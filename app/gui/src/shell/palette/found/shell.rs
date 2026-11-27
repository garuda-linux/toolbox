//! The palette's own rows: the pages, the shell's commands and the
//! Maintenance page's operations, tools and repairs.

use egui::Context;
use toolbox_core::features::search::Needle;
use toolbox_core::features::search::ranked::top;
use toolbox_gui_kit::icons;

use super::{Look, Part, Pick, Row, Section};
use crate::command::{MaintenanceCommand, UiCommand};
use crate::i18n;
use crate::page::PageId;
use crate::page_ui::{PageLook as _, listed};
use crate::pages::{PageContext, maintenance};

pub(super) fn page_look(page: PageId) -> Look {
    Look::Glyph(page.icon())
}

/// The section and tile of a command. A command's glyph is the one its
/// meaning has elsewhere in the app, Help's the guide's.
fn command_look(command: UiCommand) -> (Section, Look) {
    let glyph = match command {
        UiCommand::Navigate(page) => return (Section::Pages, page_look(page)),
        UiCommand::Help => return (Section::Commands, page_look(PageId::Guide)),
        UiCommand::Apply => icons::SYSTEM_SOFTWARE_INSTALL,
        UiCommand::ToggleSidebar => icons::SIDEBAR_COLLAPSE_LEFT,
        UiCommand::ToggleConsole => icons::UTILITIES_TERMINAL,
        UiCommand::OpenPalette | UiCommand::FocusSearch => icons::SYSTEM_SEARCH,
        UiCommand::ReviewChanges | UiCommand::ClearChanges => icons::VIEW_PIM_TASKS,
        UiCommand::ZoomIn => icons::ZOOM_IN,
        UiCommand::ZoomOut => icons::ZOOM_OUT,
        UiCommand::ZoomReset => icons::ZOOM_ORIGINAL,
        UiCommand::Quit => icons::WINDOW_CLOSE,
    };

    (Section::Commands, Look::Glyph(glyph))
}

pub fn all(ctx: &Context, context: &PageContext<'_>) -> Vec<Row> {
    let pages = PageId::ALL
        .into_iter()
        .filter(|page| listed(ctx, *page))
        .map(UiCommand::Navigate);

    // Reviewing has nothing to show while the queue is empty.
    let review = (!context.changes.is_empty()).then_some(UiCommand::ReviewChanges);

    pages
        .chain(UiCommand::ACTIONS)
        .chain(review)
        .filter(|command| *command != UiCommand::OpenPalette)
        .map(|command| {
            let (section, look) = command_look(command);
            Row {
                name: i18n::text(ctx, command.text_id()).into_owned(),
                detail: String::new(),
                shortcut: command
                    .shortcut()
                    .map(|shortcut| ctx.format_shortcut(&shortcut)),
                section,
                look,
                pick: Pick::Run(command.into()),
            }
        })
        .collect()
}

pub fn found(ctx: &Context, context: &PageContext<'_>, needle: &Needle) -> [Option<Part>; 2] {
    let (pages, commands): (Vec<Row>, Vec<Row>) = all(ctx, context)
        .into_iter()
        .partition(|row| row.section == Section::Pages);

    <[Vec<Row>; 2]>::from((pages, commands)).map(|rows| {
        let ranked = rows
            .into_iter()
            .filter_map(|row| needle.rank(&row.name, []).map(|rank| (rank, row)));

        top(ranked, usize::MAX).map(Part::here)
    })
}

pub fn maintenance(ctx: &Context, needle: &Needle) -> Option<Part> {
    let rows = maintenance::palette::entries()
        .into_iter()
        .map(|(id, tab)| Row {
            name: i18n::text(ctx, id).into_owned(),
            detail: i18n::text(ctx, tab.text_id()).into_owned(),
            shortcut: None,
            section: Section::Maintenance,
            look: page_look(PageId::Maintenance),
            pick: Pick::Run(MaintenanceCommand::Reveal(tab).into()),
        });

    let ranked = rows.filter_map(|row| needle.rank(&row.name, []).map(|rank| (rank, row)));
    top(ranked, usize::MAX).map(Part::here)
}
