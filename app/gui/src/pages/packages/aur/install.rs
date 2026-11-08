//! The question before an AUR build: the package is built from a
//! community script on this computer, and paru shows that script in the
//! Console before it builds.

use egui::{Context, Id};
use toolbox_core::Toggle;
use toolbox_gui_kit::widgets::{ButtonKind, DialogWidth, Question, question};

use super::super::PackagesPage;
use crate::command::PackagesCommand;
use crate::i18n;
use crate::pages::PageContext;

pub fn dialog(page: &mut PackagesPage, ctx: &Context, context: &PageContext<'_>) {
    let Some(package) = &page.installing else {
        return;
    };

    let title = i18n::text_with(
        ctx,
        "packages-aur-install-title",
        &[("package", package.as_str())],
    );

    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text(ctx, "packages-aur-install-body");
    let cancel = i18n::text(ctx, "packages-cancel");
    let confirm = i18n::text(ctx, "packages-install");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Primary,
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
