//! Unloading a driver stops the devices it runs at once, the connection
//! included when it is the network card's, so it asks first. Which device
//! a module runs is not read, so every unload asks.

use egui::{Context, Id};
use toolbox_gui_kit::widgets::{self, ButtonKind, DialogWidth, Pending, Question};
use toolbox_plan::KernelModule;

use crate::command::NetworkCommand;
use crate::i18n;
use crate::pages::PageContext;

fn asking() -> Pending<KernelModule> {
    Pending::new(Id::unique("toolbox_gui::network::unload"))
}

/// Drops an open question, as when the page is left.
pub fn forget(ctx: &Context) {
    asking().release(ctx);
}

pub fn ask(ctx: &Context, module: &KernelModule) {
    asking().hold(ctx, module.clone());
}

pub fn question(ctx: &Context, context: &PageContext<'_>) {
    let Some(module) = asking().get(ctx) else {
        return;
    };

    let name = module.as_str();
    let title = i18n::text_with(ctx, "network-driver-unload-title", &[("module", name)]);
    let close = i18n::text(ctx, "dialog-close");
    let body = i18n::text_with(ctx, "network-driver-unload-question", &[("module", name)]);
    let cancel = i18n::text(ctx, "network-driver-unload-keep");
    let confirm = i18n::text(ctx, "network-driver-unload");
    let texts = Question {
        title: &title,
        close: &close,
        body: &body,
        cancel: &cancel,
        confirm: &confirm,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = widgets::question(ctx, asking().dialog(), texts);
    if asked.confirmed {
        context.commands.send(NetworkCommand::LoadDriver {
            module,
            load: false,
        });
    }

    if asked.done {
        asking().release(ctx);
    }
}
