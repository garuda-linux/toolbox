use egui::{Context, Label, Ui};
use toolbox_cli::output::size;
use toolbox_core::Toggle;
use toolbox_core::features::packages::list::{self, Catalog, row_state};
use toolbox_core::features::packages::{IndexEntry, InstallReason, Origin};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowState, Tone, named_pill_button, pill_width, rich, row_height,
    split_row, status_dot, tag, text_block_height, title_line,
};
use toolbox_plan::PackageName;

use super::{PackagesPage, remove};
use crate::command::PackagesCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{Initial, package_picture, picture_height, picture_slot};

const LINES: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Install,
    Remove,
    TakeBack(Toggle),
}

/// What the shared row, list and detail header need of a repository or
/// an AUR row.
pub trait PackageRow: ListItem + Copy {
    /// The state word of a queued install, which for the AUR is a build.
    const INSTALLS: &'static str;

    fn package(&self) -> &PackageName;

    fn state(&self, context: &PageContext<'_>) -> list::RowState;

    fn act(&self, page: &mut PackagesPage, context: &PageContext<'_>, act: Act);

    /// The tags and marks between the name and the state.
    fn marks(&self, ui: &mut Ui);

    fn description(&self) -> &str;

    /// The row's second small line, under the description.
    fn detail_line(&self, ctx: &Context) -> String;

    /// The line under the detail page's title, before the state.
    fn header_line(&self, ctx: &Context) -> String;
}

/// One shown package of the index.
#[derive(Clone, Copy, Debug)]
pub struct Row<'a> {
    pub entry: &'a IndexEntry,
    pub catalog: &'a Catalog,
}

impl ListItem for Row<'_> {
    fn key(&self) -> &str {
        self.entry.name.as_str()
    }

    fn name(&self) -> &str {
        self.entry.name.as_str()
    }
}

impl PackageRow for Row<'_> {
    const INSTALLS: &'static str = "packages-installs";

    fn package(&self) -> &PackageName {
        &self.entry.name
    }

    fn state(&self, context: &PageContext<'_>) -> list::RowState {
        row_state(&self.entry.name, context.snapshot, context.changes)
    }

    fn act(&self, page: &mut PackagesPage, context: &PageContext<'_>, act: Act) {
        let package = self.entry.name.clone();
        match act {
            Act::Install => context.commands.send(PackagesCommand::Toggle {
                package,
                wish: Toggle::On,
            }),
            Act::Remove => page.removing = Some(remove::Removal::of(package, self.catalog)),
            Act::TakeBack(wish) => context.commands.send(PackagesCommand::Toggle {
                package,
                wish: undone(wish),
            }),
        }
    }

    fn marks(&self, ui: &mut Ui) {
        for id in tags(self.entry) {
            tag(ui, &i18n::text(ui.ctx(), id), Tone::Neutral);
        }
    }

    fn description(&self) -> &str {
        &self.entry.description
    }

    fn detail_line(&self, ctx: &Context) -> String {
        origin_line(ctx, self.entry)
    }

    fn header_line(&self, ctx: &Context) -> String {
        let entry = self.entry;
        format!("{} · {}", entry.version, repository(ctx, &entry.origin))
    }
}

/// The picture or the text, whichever is taller.
pub fn height(ui: &Ui) -> f32 {
    row_height(
        ui,
        text_block_height(ui, LINES).max(picture_height(ui, false)),
    )
}

/// Draws the row. Returns whether it asked to open the detail view: a
/// click beside the button, or Space on the focused row.
pub fn show<R>(
    ui: &mut Ui,
    context: &PageContext<'_>,
    page: &mut PackagesPage,
    row: &R,
    state: &RowState,
) -> bool
where
    R: PackageRow,
{
    let button = button_text(ui, context, row);
    let width = pill_width(ui, &button.0);

    split_row(
        ui,
        width,
        |ui| {
            ui.horizontal_top(|ui| {
                let at = picture_slot(ui, false);
                ui.vertical(|ui| text(ui, context, row));
                let package = row.package();
                package_picture(
                    ui,
                    context.pictures,
                    Some(package),
                    Initial::named(package),
                    at,
                );
            });
        },
        |ui| action(ui, page, context, row, &button),
    );

    state.toggle
}

/// The button's text and what it does.
pub fn button_text<R>(ui: &Ui, context: &PageContext<'_>, row: &R) -> (String, Act)
where
    R: PackageRow,
{
    let state = row.state(context);
    let (id, act) = match state.queued {
        Some(wish) => ("packages-take-back", Act::TakeBack(wish)),
        None if state.installed => ("packages-remove", Act::Remove),
        None => ("packages-install", Act::Install),
    };

    (i18n::text(ui.ctx(), id).into_owned(), act)
}

/// The row's and the detail header's button. Its name carries the
/// package, as several rows show the same button text.
pub fn action<R>(
    ui: &mut Ui,
    page: &mut PackagesPage,
    context: &PageContext<'_>,
    row: &R,
    (text, act): &(String, Act),
) where
    R: PackageRow,
{
    let name = i18n::text_with(
        ui.ctx(),
        "packages-action-name",
        &[("action", text), ("name", row.package().as_str())],
    );

    let clicked = ui
        .add_enabled_ui(!context.busy, |ui| {
            named_pill_button(ui, text, ButtonKind::Soft, &name)
        })
        .inner
        .clicked();

    if clicked {
        row.act(page, context, *act);
    }
}

/// What waits for Apply, else whether it is installed.
pub fn state_word<R>(context: &PageContext<'_>, row: &R) -> Option<(&'static str, Tone)>
where
    R: PackageRow,
{
    let state = row.state(context);
    match state.queued {
        Some(Toggle::On) => Some((R::INSTALLS, Tone::Accent)),
        Some(Toggle::Off) => Some(("packages-removes", Tone::Accent)),
        None if state.installed => Some(("packages-installed", Tone::Success)),
        None => None,
    }
}

/// Taking back a queued change queues its opposite.
pub const fn undone(wish: Toggle) -> Toggle {
    if wish.is_on() {
        Toggle::Off
    } else {
        Toggle::On
    }
}

fn text<R>(ui: &mut Ui, context: &PageContext<'_>, row: &R)
where
    R: PackageRow,
{
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);

    title_line(ui, false, |ui| {
        ui.add(
            Label::new(rich(
                ui,
                row.package().as_str(),
                Weight::SemiBold,
                BODY,
                tokens.text,
            ))
            .truncate(),
        );

        row.marks(ui);
        if let Some((id, tone)) = state_word(context, row) {
            status_dot(ui, &i18n::text(&ctx, id), tone);
        }
    });

    // The two small lines stack as one block, as the row's height counts
    // them.
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for line in [row.description().to_owned(), row.detail_line(&ctx)] {
            ui.add(
                Label::new(rich(ui, &line, Weight::Regular, SMALL, tokens.secondary)).truncate(),
            );
        }
    });
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

pub fn repository(ctx: &Context, origin: &Origin) -> String {
    match origin {
        Origin::Repository(repo) => repo.clone(),
        Origin::Foreign => i18n::text(ctx, "packages-no-repository").into_owned(),
    }
}
