//! The question before an AUR build: the package is built from a
//! community script on this computer, and the Console shows that script
//! before it builds, paru's on the old path and the app's review otherwise.

use egui::{Context, Id};
use toolbox_core::Toggle;
use toolbox_core::features::packages::aur::takes_split_path;
use toolbox_gui_kit::widgets::{ButtonKind, DialogWidth, Question, question};

use super::super::PackagesPage;
use crate::command::PackagesCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;

pub fn dialog(page: &mut PackagesPage, ctx: &Context, context: &PageContext<'_>) {
    let Some(package) = &page.installing else {
        return;
    };

    let title = i18n::text_with(
        ctx,
        "packages-aur-install-title",
        &[("package", Arg::from(package.as_str()))],
    );

    let close = i18n::text(ctx, "dialog-close");
    let split = takes_split_path(context.snapshot);

    let body = if split {
        i18n::text(ctx, "packages-aur-install-body-split")
    } else {
        i18n::text(ctx, "packages-aur-install-body")
    };

    let cancel = i18n::text(ctx, "packages-cancel");
    let confirm = i18n::text(ctx, "packages-aur-install-confirm");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Primary,
        icon: None,
        width: DialogWidth::Small,
    };

    let asked = question(
        ctx,
        Id::unique("toolbox_gui::packages::aur::install"),
        texts,
    );

    if asked.confirmed {
        context.commands.send(PackagesCommand::ToggleAur {
            package: package.clone(),
            wish: Toggle::On,
        });
    }

    if asked.done {
        page.installing = None;
    }
}
