mod text;

use std::borrow::Cow;

use egui::{Context, Sense, Ui, Vec2};
pub use text::repository;
use text::{optional_tag, origin_line, tags, text};
use toolbox_core::Toggle;
use toolbox_core::features::packages::IndexEntry;
use toolbox_core::features::packages::list::{self, Catalog, Filter, row_state};
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowState, Tone, named_pill_button, pill_width, push_height, row_height,
    split_row_in, tag_on_row, text_block_height, text_block_height_with,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::PackageName;

use super::{PackagesPage, remove};
use crate::artwork::Artwork;
use crate::command::PackagesCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::{Initial, package_picture, picture_height, picture_slot};

/// The title and the two small lines of a full-width row: the description
/// and the origin line.
const LINES: u8 = 2;
/// The title and the description of a row beside the details.
const COMPACT_LINES: u8 = 1;
const INSTALLED: &str = "packages-installed";
/// As large as a link row's chevron.
const CHEVRON: f32 = 14.0;

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

    fn marks(&self, ui: &mut Ui, selected: bool);

    fn description(&self) -> &str;

    fn detail_line(&self, ctx: &Context) -> String;

    fn header_line(&self, ctx: &Context) -> String;
}

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

    fn marks(&self, ui: &mut Ui, selected: bool) {
        for id in tags(self.entry) {
            tag_on_row(ui, &i18n::text(ui.ctx(), id), selected);
        }

        optional_tag(ui, self.catalog.optional_for(&self.entry.name), selected);
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

/// A full-width row keeps its height with its controls, and the row beside
/// the details has one line less, as its controls are in the details.
pub fn height(ui: &Ui, compact: bool) -> f32 {
    let content = if compact {
        text_block_height(ui, COMPACT_LINES)
    } else {
        text_block_height_with(ui, LINES, push_height(ui))
    };

    row_height(ui, content.max(picture_height(ui, false)))
}

/// Whether the picked chip lists installed packages only, so their state
/// word would repeat it.
const fn says_installed(filter: Filter) -> bool {
    matches!(
        filter,
        Filter::Installed | Filter::Updates | Filter::Orphans | Filter::Optional
    )
}

pub fn button_text<R>(ui: &Ui, context: &PageContext<'_>, row: &R) -> (Cow<'static, str>, Act)
where
    R: PackageRow,
{
    button_of(ui.ctx(), row.state(context))
}

fn button_of(ctx: &Context, state: list::RowState) -> (Cow<'static, str>, Act) {
    let (id, act) = match state.queued {
        Some(wish) => ("packages-take-back", Act::TakeBack(wish)),
        None if state.installed => ("packages-remove", Act::Remove),
        None => ("packages-install", Act::Install),
    };

    (i18n::text(ctx, id), act)
}

/// Draws the row. Full width it keeps the install or remove button and the
/// chevron that says it opens. Beside the details it has no button, as the
/// details hold it, and the arrow keys open a row they reach. Returns whether
/// the row asked to open its details.
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
    let standing = row.state(context);

    // The installed chips say it of every row already.
    let word =
        word_of::<R>(standing).filter(|(id, _)| !(*id == INSTALLED && says_installed(page.filter)));

    if page.open.is_some() {
        text_and_picture(ui, context, row, word, (state.selected, false));
        return state.toggle || state.keyed;
    }

    let button = button_of(ui.ctx(), standing);
    let width = pill_width(ui, &button.0) + ui.spacing().item_spacing.x + CHEVRON;
    split_row_in(
        ui,
        state.row,
        width,
        |ui| text_and_picture(ui, context, row, word, (false, true)),
        |ui| {
            if action(ui, context, row, &button.0, ButtonKind::Soft) {
                row.act(page, context, button.1);
            }

            chevron(ui);
        },
    );

    state.toggle
}

fn text_and_picture<R>(
    ui: &mut Ui,
    context: &PageContext<'_>,
    row: &R,
    word: Option<(&'static str, Tone)>,
    (selected, origin): (bool, bool),
) where
    R: PackageRow,
{
    ui.horizontal_top(|ui| {
        let at = picture_slot(ui, false);
        ui.vertical(|ui| text(ui, row, word, selected, origin));
        picture(ui, context, row, at);
    });
}

fn picture<R>(ui: &Ui, context: &PageContext<'_>, row: &R, at: egui::Rect)
where
    R: PackageRow,
{
    let package = row.package();
    package_picture(
        ui,
        context.service::<Artwork>().pictures(),
        Some(package),
        Initial::named(package),
        at,
    );
}

/// Tells that the row opens the package's page, as a link row's does.
/// The row itself is the target, so the glyph takes no click.
fn chevron(ui: &mut Ui) {
    let ink = tokens::current(ui.ctx()).secondary;
    let (at, _) = ui.allocate_exact_size(Vec2::splat(CHEVRON), Sense::hover());
    let theme = icons::theme_in_use(ui.ctx());
    icons::GO_NEXT.image(theme.as_deref(), ink).paint_at(ui, at);
}

/// The row's and the detail header's button. Its name carries the
/// package, as several rows show the same button text.
#[must_use = "the caller holds the page, so it acts on the press"]
pub fn action<R>(
    ui: &mut Ui,
    context: &PageContext<'_>,
    row: &R,
    text: &str,
    kind: ButtonKind,
) -> bool
where
    R: PackageRow,
{
    let name = i18n::text_with(
        ui.ctx(),
        "packages-action-name",
        &[
            ("action", Arg::from(text)),
            ("name", Arg::from(row.package().as_str())),
        ],
    );

    ui.enabled_ui(!context.busy, |ui| named_pill_button(ui, text, kind, &name))
        .inner
        .clicked()
}

/// What waits for Apply, else whether it is installed.
pub fn state_word<R>(context: &PageContext<'_>, row: &R) -> Option<(&'static str, Tone)>
where
    R: PackageRow,
{
    word_of::<R>(row.state(context))
}

const fn word_of<R>(state: list::RowState) -> Option<(&'static str, Tone)>
where
    R: PackageRow,
{
    match state.queued {
        Some(Toggle::On) => Some((R::INSTALLS, Tone::Accent)),
        Some(Toggle::Off) => Some(("packages-removes", Tone::Accent)),
        None if state.installed => Some((INSTALLED, Tone::Success)),
        None => None,
    }
}

pub const fn undone(wish: Toggle) -> Toggle {
    if wish.is_on() {
        Toggle::Off
    } else {
        Toggle::On
    }
}
