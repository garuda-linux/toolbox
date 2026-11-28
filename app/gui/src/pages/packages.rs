//! The Packages page: every repository package and every installed one,
//! filtered by chips and a search, with a detail page per package.
//!
//! Install and Remove queue the package like Discover's apps do. With the
//! setting on, the AUR chip lists the AUR's packages apart from the
//! repositories'.

mod aur;
mod detail;
mod list;
mod remove;
mod row;
pub use row::repository;
#[cfg(test)]
mod tests;

use egui::{Key, Modifiers, Ui};
use toolbox_core::features::packages::aur::AurCatalog;
use toolbox_core::features::packages::list::{Filter, View};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight, font};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::columns::{self, Columns};
use toolbox_plan::PackageName;

use super::{PageContext, PageView, place_of, plan_running};
use crate::command::PackagesCommand;

#[derive(Debug, Default)]
pub struct PackagesPage {
    search: String,
    filter: Filter,
    /// The package the detail page shows, `None` for the list.
    open: Option<PackageName>,
    open_aur: bool,
    /// The packages the detail page came through, each with whether it
    /// showed its AUR side, the first one opened from the list first. Back
    /// returns to the last of them.
    trail: Vec<(PackageName, bool)>,
    /// Whether the page is too narrow for the list and the details side by
    /// side, the details then replace the list.
    narrow: bool,
    unfolded: Unfolded,
    view: RowCache<ViewKey, View>,
    aur_view: RowCache<Same<AurCatalog>, Vec<usize>>,
    history: detail::HistoryRows,
    facts: detail::FactRows,
    targets: detail::TargetRows,
    file_filter: String,
    files: detail::FileRows,
    aur_asked: bool,
    removing: Option<remove::Removal>,
    installing: Option<PackageName>,
    back_to: Option<PackageName>,
}

/// What the reader opened on the detail page. Every package starts with
/// its lists cut, the files folded and the history open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each part of the page folds on its own"
)]
pub struct Unfolded {
    pub depends: bool,
    pub libraries: bool,
    pub make_depends: bool,
    pub required_by: bool,
    pub optional: bool,
    pub files: bool,
    pub history_folded: bool,
    pub history_all: bool,
}

/// What a view comes from besides the search: the filter, the index and
/// the status reading that holds the pending updates.
type ViewKey = (
    Filter,
    Same<toolbox_core::features::packages::list::Catalog>,
    u64,
);

impl PageView for PackagesPage {
    /// The list and the details each scroll on their own, so the page
    /// body has no scroll area of its own.
    fn shape(&self) -> toolbox_gui_kit::widgets::BodyShape {
        toolbox_gui_kit::widgets::BodyShape::Panes {
            width: f32::INFINITY,
        }
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        list::header(self, ui, context);
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        plan_running(ui, context);
        list::refusal(ui, context);
        list::top(self, ui, context);
        ui.add_space(ui.tokens().sizes.header_gap);

        if self.escape_back(ui) {
            self.back(context);
        }

        self.narrow = columns::is_narrow(ui.available_width());
        let open = self.open.is_some();
        Columns::new("packages-columns").open(open).show(
            ui,
            self,
            |ui, page| list::column(page, ui, context),
            |ui, page| detail::body(page, ui, context),
        );

        remove::dialog(self, ui.ctx(), context);
        aur::install::dialog(self, ui.ctx(), context);
    }

    fn place(&self) -> u64 {
        place_of(&self.open)
    }
}

impl PackagesPage {
    /// Back returns to the list searched for the package.
    pub fn reveal(&mut self, package: &PackageName, aur: bool) {
        self.search_for(package.as_str(), aur);
        self.opens(package, aur);
    }

    pub fn search_for(&mut self, text: &str, aur: bool) {
        text.clone_into(&mut self.search);
        self.filter = if aur { Filter::Aur } else { Filter::All };
        self.open = None;
        self.trail.clear();
    }

    fn opens(&mut self, package: &PackageName, aur: bool) {
        self.open = Some(package.clone());
        self.open_aur = aur;
        self.unfolded = Unfolded::default();
        self.file_filter.clear();
    }

    /// Opens the detail page of `package` from the list, its AUR side for
    /// a row of the AUR chip.
    fn open(&mut self, package: &PackageName, aur: bool, context: &PageContext<'_>) {
        // The arrow keys open a row before Space or a click asks for it again.
        if self.open.as_ref() == Some(package) && self.open_aur == aur {
            return;
        }

        self.trail.clear();
        self.show(package, aur, context);
    }

    /// Opens the detail page of `package` from the one that shows, which
    /// Back returns to.
    fn follow(&mut self, package: &PackageName, aur: bool, context: &PageContext<'_>) {
        if let Some(from) = self.open.take() {
            self.trail.push((from, self.open_aur));
        }

        self.show(package, aur, context);
    }

    /// The open package as the list of its side shows it, the AUR's or the
    /// repositories'. Another side's list has no selected row.
    fn selection(&self, aur: bool) -> Option<String> {
        self.open
            .as_ref()
            .filter(|_| self.open_aur == aur)
            .map(|package| package.as_str().to_owned())
    }

    /// Escape returns from the details like Back does, unless a question
    /// of the page or a dialog over it, such as the command palette, is
    /// open. Each answers Escape itself.
    fn escape_back(&self, ui: &Ui) -> bool {
        self.open.is_some()
            && self.removing.is_none()
            && self.installing.is_none()
            && ui.memory(|memory| memory.top_modal_layer().is_none())
            && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
    }

    /// Back to the package the detail page came from, else to the list
    /// with the row of the first package focused again.
    fn back(&mut self, context: &PageContext<'_>) {
        match self.trail.pop() {
            Some((previous, aur)) => self.show(&previous, aur, context),
            None => self.back_to = self.open.take(),
        }
    }

    fn show(&mut self, package: &PackageName, aur: bool, context: &PageContext<'_>) {
        self.opens(package, aur);
        context
            .commands
            .send(PackagesCommand::Show(package.clone()));
    }
}

const fn shows_aur(context: &PageContext<'_>) -> bool {
    context.preferences.settings.packages.show_aur
}

/// A wrapped line of text and links as tall as a link's line, so the
/// text sits where a plain label would. egui's row is otherwise
/// `interact_size` tall, nearly a pill.
fn text_line(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let link = font(ui.style(), Weight::Medium, BODY);
    let height = ui.fonts_mut(|fonts| fonts.row_height(&link));
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = height;
        ui.horizontal_wrapped(add);
    });
}
