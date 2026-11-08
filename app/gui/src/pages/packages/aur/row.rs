//! One AUR package: its picture, its name with the AUR tag, an out of
//! date mark and its state, one line of its description, its version and
//! votes, and Install or Remove.

use egui::{Context, Ui};
use toolbox_core::features::packages::aur::{AurPackage, row_state};
use toolbox_core::features::packages::list::{Catalog, RowState};
use toolbox_gui_kit::widgets::{ListItem, Tone, status_dot, tag};
use toolbox_plan::PackageName;

use super::super::row::{Act, PackageRow, undone};
use super::super::{PackagesPage, remove};
use crate::command::PackagesCommand;
use crate::i18n;
use crate::pages::PageContext;

/// One shown package of the AUR's copy, with the repositories' index for
/// what a removal takes along.
#[derive(Clone, Copy, Debug)]
pub struct AurRow<'a> {
    pub entry: &'a AurPackage,
    pub repo: &'a Catalog,
}

impl ListItem for AurRow<'_> {
    fn key(&self) -> &str {
        self.entry.name.as_str()
    }

    fn name(&self) -> &str {
        self.entry.name.as_str()
    }
}

impl PackageRow for AurRow<'_> {
    const INSTALLS: &'static str = "packages-aur-builds";

    fn package(&self) -> &PackageName {
        &self.entry.name
    }

    fn state(&self, context: &PageContext<'_>) -> RowState {
        row_state(&self.entry.name, context.snapshot, context.changes)
    }

    /// Install asks first, as a build runs a script from the AUR. Remove
    /// asks as for any installed package.
    fn act(&self, page: &mut PackagesPage, context: &PageContext<'_>, act: Act) {
        let package = self.entry.name.clone();
        match act {
            Act::Install => page.installing = Some(package),
            Act::Remove => page.removing = Some(remove::Removal::of(package, self.repo)),
            Act::TakeBack(wish) => context.commands.send(PackagesCommand::ToggleAur {
                package,
                wish: undone(wish),
            }),
        }
    }

    fn marks(&self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        tag(ui, &i18n::text(&ctx, "packages-tag-aur"), Tone::Neutral);
        if self.entry.out_of_date.is_some() {
            status_dot(
                ui,
                &i18n::text(&ctx, "packages-aur-out-of-date"),
                Tone::Warning,
            );
        }
    }

    fn description(&self) -> &str {
        &self.entry.description
    }

    fn detail_line(&self, ctx: &Context) -> String {
        version_line(ctx, self.entry)
    }

    fn header_line(&self, ctx: &Context) -> String {
        let aur = i18n::text(ctx, "packages-tag-aur");
        format!("{aur} · {}", version_line(ctx, self.entry))
    }
}

/// "1.0-1 · 42 votes". The tag beside the name says where it comes from.
pub fn version_line(ctx: &Context, entry: &AurPackage) -> String {
    let votes = i18n::text_with(
        ctx,
        "packages-aur-votes",
        &[("count", &entry.votes.to_string())],
    );

    format!("{} · {votes}", entry.version)
}
