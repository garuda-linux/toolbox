//! What the palette finds for a text: every page and command by name,
//! then the Maintenance page's operations, tools and repairs, then the
//! Discover apps whose title holds it. Each row knows its section and the
//! tile it shows.

use std::rc::Rc;
use std::sync::Arc;

use egui::Context;
use fluent_templates::LanguageIdentifier;
use toolbox_core::Snapshot;
use toolbox_core::features::discover::package;
use toolbox_core::features::search::Needle;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::tokens::Hue;
use toolbox_plan::PackageName;

use crate::command::{Command, DiscoverCommand, MaintenanceCommand, UiCommand};
use crate::i18n;
use crate::page::PageId;
use crate::page_ui::{GroupLook as _, PageLook as _, listed};
use crate::pages::discover::category_title;
use crate::pages::{PageContext, maintenance};

/// At most this many rows: a short text matches hundreds of apps.
const LIMIT: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Pages,
    Commands,
    Maintenance,
    Apps,
}

#[derive(Clone, Debug)]
pub enum Look {
    Glyph(Icon, Hue),
    /// The app's letter tile. `package` keeps its hue, as on Discover.
    Letter {
        package: String,
    },
}

#[derive(Debug)]
pub struct Row {
    pub name: String,
    pub detail: String,
    /// The shortcut as the platform writes it, drawn as keycaps.
    pub shortcut: Option<String>,
    pub section: Section,
    pub look: Look,
    pub command: Command,
}

/// What the found rows depend on, so the palette finds them again only
/// when one of these changes and not on every frame it is open.
#[derive(Debug)]
pub struct Asked {
    query: String,
    revision: u64,
    language: LanguageIdentifier,
    /// Held, so a new snapshot is told from the last by identity.
    snapshot: Arc<Snapshot>,
    boot_repair: bool,
}

impl Asked {
    fn of(ctx: &Context, context: &PageContext<'_>, query: &str) -> Self {
        Self {
            query: query.to_owned(),
            revision: context.revision,
            language: i18n::language(ctx),
            snapshot: Arc::clone(context.snapshot),
            boot_repair: listed(ctx, PageId::BootRepair),
        }
    }

    fn same(&self, other: &Self) -> bool {
        self.query == other.query
            && self.revision == other.revision
            && self.language == other.language
            && Arc::ptr_eq(&self.snapshot, &other.snapshot)
            && self.boot_repair == other.boot_repair
    }
}

impl Section {
    pub fn label(self, ctx: &Context) -> String {
        let id = match self {
            Self::Pages => "palette-group-pages",
            Self::Commands => "palette-group-commands",
            Self::Maintenance => PageId::Maintenance.text_id(),
            Self::Apps => "palette-group-apps",
        };

        i18n::text(ctx, id).into_owned()
    }
}

fn page_look(page: PageId) -> Look {
    Look::Glyph(page.icon(), page.group().hue())
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
        UiCommand::Quit => icons::WINDOW_CLOSE,
    };

    (Section::Commands, Look::Glyph(glyph, Hue::Accent))
}

/// The rows for `query`, kept in `found` while nothing they depend on
/// differs.
pub fn cached(
    found: &mut Option<(Asked, Rc<[Row]>)>,
    ctx: &Context,
    context: &PageContext<'_>,
    query: &str,
) -> Rc<[Row]> {
    let asked = Asked::of(ctx, context, query);
    if let Some((before, rows)) = found.as_ref()
        && before.same(&asked)
    {
        return Rc::clone(rows);
    }

    let rows: Rc<[Row]> = matches(ctx, context, query).into();
    *found = Some((asked, Rc::clone(&rows)));
    rows
}

/// Every page and command whose name holds the typed text, then the
/// Maintenance entries and the apps this system offers whose title holds
/// it. Those two only show for a typed text, there are too many to list.
fn matches(ctx: &Context, context: &PageContext<'_>, query: &str) -> Vec<Row> {
    let needle = Needle::new(query);
    let pages = PageId::ALL
        .into_iter()
        .filter(|page| listed(ctx, *page))
        .map(UiCommand::Navigate);

    // Reviewing has nothing to show while the queue is empty.
    let review = (!context.changes.is_empty()).then_some(UiCommand::ReviewChanges);
    let commands = pages
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
                command: command.into(),
            }
        })
        .filter(|row| needle.is_in(&row.name));

    let maintenance = (!needle.is_empty())
        .then(maintenance::palette::entries)
        .into_iter()
        .flatten()
        .map(|(id, tab)| Row {
            name: i18n::text(ctx, &id).into_owned(),
            detail: i18n::text(ctx, tab.text_id()).into_owned(),
            shortcut: None,
            section: Section::Maintenance,
            look: page_look(PageId::Maintenance),
            command: MaintenanceCommand::Reveal(tab).into(),
        })
        .filter(|row| needle.is_in(&row.name));

    let apps = app_rows(ctx, context, &needle);

    commands
        .chain(maintenance)
        .chain(apps)
        .take(LIMIT)
        .collect()
}

/// The apps this system offers whose title holds `needle`, none for an
/// empty one.
fn app_rows<'a>(
    ctx: &'a Context,
    context: &'a PageContext<'_>,
    needle: &'a Needle,
) -> impl Iterator<Item = Row> + 'a {
    (!needle.is_empty())
        .then(|| {
            context
                .catalogs
                .apps
                .apps
                .offered(context.snapshot, context.changes)
        })
        .into_iter()
        .flatten()
        .filter(|(_, hit)| hit.title_matches(needle))
        .map(|(_, hit)| {
            let title = hit.entry.title();
            let kept = package(hit.entry).map_or_else(|| hit.entry.id(), PackageName::as_str);
            Row {
                name: title.to_owned(),
                detail: category_title(ctx, hit.category.id(), hit.category.title()),
                shortcut: None,
                section: Section::Apps,
                look: Look::Letter {
                    package: kept.to_owned(),
                },
                command: DiscoverCommand::Reveal(title.to_owned()).into(),
            }
        })
}
