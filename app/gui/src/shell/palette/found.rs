//! Without a text the palette lists every page and command. With one each
//! source shows its best rows, and the section with the best row comes first.

mod apps;
mod packages;
mod shell;

use std::rc::Rc;
use std::sync::Arc;

use egui::Context;
use fluent_templates::LanguageIdentifier;
use toolbox_core::Snapshot;
use toolbox_core::features::search::Needle;
use toolbox_core::features::search::ranked::{Ranked, by_best};
use toolbox_gui_kit::icons::Icon;

use crate::command::Command;
use crate::i18n;
use crate::i18n::Arg;
use crate::page::PageId;
use crate::page_ui::listed;
use crate::pages::PageContext;

const CAP: usize = 5;

/// In the order sections take when their best rows fit alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Pages,
    Commands,
    Maintenance,
    Apps,
    Packages,
    Aur,
}

#[derive(Clone, Debug)]
pub enum Look {
    Glyph(Icon),
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
    pub pick: Pick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    Run(Command),
    Expand(Section),
}

#[derive(Debug)]
enum Rest {
    /// The section ranked only its best few.
    Page(PageId, Command),
    Here,
}

pub struct Part {
    ranked: Ranked<Row>,
    rest: Rest,
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
    packages: packages::Lists,
    expanded: Vec<Section>,
}

impl Asked {
    fn of(ctx: &Context, context: &PageContext<'_>, query: &str, expanded: &[Section]) -> Self {
        Self {
            query: query.to_owned(),
            expanded: expanded.to_vec(),
            revision: context.revision,
            language: i18n::language(ctx),
            snapshot: Arc::clone(context.snapshot),
            boot_repair: listed(ctx, PageId::BootRepair),
            packages: packages::lists(context),
        }
    }

    fn same(&self, other: &Self) -> bool {
        self.query == other.query
            && self.expanded == other.expanded
            && self.revision == other.revision
            && self.language == other.language
            && Arc::ptr_eq(&self.snapshot, &other.snapshot)
            && self.boot_repair == other.boot_repair
            && packages::same(&self.packages, &other.packages)
    }
}

impl Section {
    pub fn label(self, ctx: &Context) -> String {
        let id = match self {
            Self::Pages => "palette-group-pages",
            Self::Commands => "palette-group-commands",
            Self::Maintenance => PageId::Maintenance.text_id(),
            Self::Apps => "palette-group-apps",
            Self::Packages => "palette-group-packages",
            Self::Aur => "palette-group-aur",
        };

        i18n::text(ctx, id).into_owned()
    }
}

impl Part {
    /// Every hit, shown capped until expanded.
    const fn here(ranked: Ranked<Row>) -> Self {
        Self {
            ranked,
            rest: Rest::Here,
        }
    }

    const fn capped(ranked: Ranked<Row>, page: PageId, command: Command) -> Self {
        Self {
            ranked,
            rest: Rest::Page(page, command),
        }
    }

    fn rows(self, ctx: &Context, expanded: &[Section]) -> Vec<Row> {
        let Self { ranked, rest } = self;
        let Some(section) = ranked.hits.first().map(|row| row.section) else {
            return Vec::new();
        };

        let total = ranked.total;
        let (cap, more) = match rest {
            Rest::Here if expanded.contains(&section) => (total, None),
            Rest::Here => {
                let text =
                    i18n::text_with(ctx, "palette-show-all-here", &[("count", Arg::from(total))]);

                let look = ranked.hits.first().map(|row| row.look.clone());

                (
                    CAP,
                    look.map(|look| (text.into_owned(), look, Pick::Expand(section))),
                )
            }
            Rest::Page(page, command) => {
                let shown = i18n::text(ctx, page.text_id());
                let args = [
                    ("count", Arg::from(total)),
                    ("page", Arg::from(shown.as_ref())),
                ];

                let text = i18n::text_with(ctx, "palette-show-all", &args).into_owned();

                (
                    CAP,
                    Some((text, shell::page_look(page), Pick::Run(command))),
                )
            }
        };

        let shown = cap.min(ranked.hits.len());
        let last = more
            .filter(|_| total > shown)
            .map(|(name, look, pick)| Row {
                name,
                detail: String::new(),
                shortcut: None,
                section,
                look,
                pick,
            });

        ranked.hits.into_iter().take(cap).chain(last).collect()
    }
}

/// Kept in `found` while nothing the rows depend on differs.
pub fn cached(
    found: &mut Option<(Asked, Rc<[Row]>)>,
    ctx: &Context,
    context: &PageContext<'_>,
    query: &str,
    expanded: &[Section],
) -> Rc<[Row]> {
    let asked = Asked::of(ctx, context, query, expanded);
    if let Some((before, rows)) = found.as_ref()
        && before.same(&asked)
    {
        return Rc::clone(rows);
    }

    let rows: Rc<[Row]> = matches(ctx, context, query, expanded).into();
    *found = Some((asked, Rc::clone(&rows)));
    rows
}

fn matches(
    ctx: &Context,
    context: &PageContext<'_>,
    query: &str,
    expanded: &[Section],
) -> Vec<Row> {
    let needle = Needle::new(query);
    if needle.is_empty() {
        return shell::all(ctx, context);
    }

    let [pages, commands] = shell::found(ctx, context, &needle);
    let [repositories, aur] = packages::found(ctx, context, &needle, query);
    let mut parts: Vec<Part> = [
        pages,
        commands,
        shell::maintenance(ctx, &needle),
        apps::found(ctx, context, &needle, query),
        repositories,
        aur,
    ]
    .into_iter()
    .flatten()
    .collect();

    by_best(&mut parts, |part| part.ranked.best);
    parts
        .into_iter()
        .flat_map(|part| part.rows(ctx, expanded))
        .collect()
}
