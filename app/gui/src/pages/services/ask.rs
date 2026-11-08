//! The question before stopping, restarting or masking a unit the session
//! lives on: the display manager, the network, polkit, D-Bus or logind.

use egui::{Context, Id};
use toolbox_gui_kit::widgets::{ButtonKind, DialogWidth, Question, question};
use toolbox_plan::{UnitName, UnitOperation, UnitScope};

use crate::command::{CommandSender, ServiceCommand};
use crate::i18n;

/// What waits for the answer.
#[derive(Clone)]
pub enum Pending {
    Control(UnitScope, UnitName, UnitOperation),
    Mask(UnitScope, UnitName),
}

impl Pending {
    const fn unit(&self) -> &UnitName {
        match self {
            Self::Control(_, unit, _) | Self::Mask(_, unit) => unit,
        }
    }

    const fn verb(&self) -> &'static str {
        match self {
            Self::Control(_, _, operation) => super::texts::operation(*operation),
            Self::Mask(..) => "services-mask",
        }
    }

    fn command(self) -> ServiceCommand {
        match self {
            Self::Control(scope, unit, operation) => ServiceCommand::Control {
                scope,
                unit,
                operation,
            },
            Self::Mask(scope, unit) => ServiceCommand::Mask {
                scope,
                unit,
                on: true,
            },
        }
    }
}

fn pending_id() -> Id {
    Id::unique("toolbox_gui::services::ask")
}

/// Holds `pending` until the page asks about it.
pub fn hold(ctx: &Context, pending: Pending) {
    ctx.data_mut(|data| data.insert_temp(pending_id(), pending));
}

/// The open question, if any. The answer sends the change or drops it.
pub fn show(ctx: &Context, commands: &CommandSender) {
    let Some(pending) = ctx.data(|data| data.get_temp::<Pending>(pending_id())) else {
        return;
    };

    let unit = pending.unit().as_str().to_owned();
    let verb = i18n::text(ctx, pending.verb());
    let title = i18n::text_with(
        ctx,
        "services-ask-title",
        &[("verb", &verb), ("unit", &unit)],
    );

    let close = i18n::text(ctx, "dialog-close");

    let why = i18n::text_with(ctx, "services-ask-why", &[("unit", &unit)]);
    let cancel = i18n::text(ctx, "services-ask-cancel");
    let texts = Question {
        title: &title,
        close: &close,
        body: &why,
        cancel: &cancel,
        confirm: &verb,
        kind: ButtonKind::Danger,
        width: DialogWidth::Small,
    };

    let asked = question(ctx, pending_id().with("dialog"), texts);
    if asked.confirmed {
        commands.send(pending.command());
    }

    if asked.done {
        ctx.data_mut(|data| data.remove::<Pending>(pending_id()));
    }
}
