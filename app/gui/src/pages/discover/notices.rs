//! What stands above the App Catalog's chips: a category that could not be
//! loaded, a refused switch and the installed packages that cannot be read.

use egui::Ui;
use toolbox_catalog::LoadProblem;
use toolbox_core::domains::InstalledPackages;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{Tone, banner};

use super::super::PageContext;
use super::super::rows::refusal_banner;
use super::super::toggles::refusal_text;
use crate::command::AppCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::page::PageId;
use crate::toggle_pages::TogglePages;

pub fn notices(ui: &mut Ui, context: &PageContext<'_>, problem: Option<&LoadProblem>) {
    let ctx = ui.ctx().clone();
    if let Some(problem) = problem {
        let reason = i18n::text(&ctx, &format!("toggles-problem-{}", problem.code));
        let text = i18n::text_with(
            &ctx,
            "discover-problem-reason",
            &[("reason", Arg::from(&reason))],
        );

        banner(ui, Tone::Warning, &text, &[]);

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    let refused = context
        .service::<TogglePages>()
        .app_refusal()
        .and_then(|refused| refusal_text(&ctx, refused));

    if let Some(text) = refused
        && refusal_banner(ui, Tone::Warning, &text, "toggles-dismiss")
    {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Discover));
    }

    // Without the installed packages every switch shows off, and switching
    // an installed app on would queue nothing.
    if context.probed && context.snapshot.part::<InstalledPackages>().is_none() {
        let again = i18n::text(&ctx, "common-read-again");
        let text = i18n::text(&ctx, "discover-unreadable");
        if banner(ui, Tone::Warning, &text, &[&again]).value.is_some() {
            context.commands.send(AppCommand::ReadSystem);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}
