//! The keyboard's way into a dialog and back: a dialog that opens takes
//! the focus, and once it is gone the focus goes back to what opened it.

use std::sync::Arc;

use egui::{Context, Id, Rect, Role, Sense, Ui, Vec2, WidgetInfo};

/// A dialog with nothing to focus stops trying after so many passes.
const TRIES: u8 = 3;

/// A dialog on screen: what had the focus when it opened, whether it
/// showed in this pass, and whether the focus still has to come in.
#[derive(Clone)]
struct Entry {
    dialog: Id,
    opener: Option<Id>,
    shown: bool,
    /// Passes left to bring the focus in, none once it is there.
    tries: u8,
}

#[derive(Clone, Default)]
struct Open(Vec<Entry>);

fn open_id() -> Id {
    Id::unique("toolbox_gui_kit::modal::focus")
}

/// Marks `dialog` as shown in this pass. A dialog that was not on screen
/// before takes the focus from what had it, onto its [`holder`]. A new
/// area sizes itself in its first pass without taking focus, so the
/// request repeats until it lands.
pub(super) fn shown(ctx: &Context, dialog: Id) {
    install(ctx);

    let focused = ctx.memory(egui::Memory::focused);
    let entry = ctx.data_mut(|data| {
        let open = data.get_temp_mut_or_default::<Open>(open_id());
        if let Some(entry) = open.0.iter_mut().find(|entry| entry.dialog == dialog) {
            entry.shown = true;
            if focused.is_some() && focused != entry.opener {
                entry.tries = 0;
            }

            entry.tries = entry.tries.saturating_sub(1);

            return entry.clone();
        }

        let entry = Entry {
            dialog,
            opener: focused,
            shown: true,
            tries: TRIES,
        };

        open.0.push(entry.clone());
        entry
    });

    if entry.tries == 0 {
        return;
    }

    ctx.memory_mut(|memory| memory.request_focus(holder_id(dialog)));
    ctx.request_repaint();
}

fn holder_id(dialog: Id) -> Id {
    dialog.with("focus_holder")
}

/// Where an opened dialog keeps the focus: on the dialog, not on a
/// control, so no focus ring shows on Close. It is a stop for Tab only
/// while the focus comes in or stays on it, so Tab moves on to Close and
/// never comes back to it.
pub(super) fn holder(ui: &Ui, dialog: Id, name: &str) {
    let id = holder_id(dialog);
    let coming_in = ui.data(|data| {
        data.get_temp::<Open>(open_id())
            .and_then(|open| open.0.into_iter().find(|entry| entry.dialog == dialog))
            .is_some_and(|entry| entry.tries > 0)
    });

    let sense = if coming_in || ui.memory(|memory| memory.has_focus(id)) {
        Sense::focusable_noninteractive()
    } else {
        Sense::hover()
    };

    let rect = Rect::from_min_size(ui.cursor().min, Vec2::ZERO);
    let response = ui.interact(rect, id, sense);
    response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, name));
}

fn install(ctx: &Context) {
    let installed = Id::unique("toolbox_gui_kit::modal::focus::installed");
    if ctx.data(|data| data.get_temp::<bool>(installed)).is_some() {
        return;
    }

    ctx.data_mut(|data| data.insert_temp(installed, true));
    ctx.on_end_pass("toolbox-dialog-focus", Arc::new(|ui| hand_back(ui.ctx())));
}

/// A dialog that did not show in this pass is gone: its opener gets the
/// focus back.
fn hand_back(ctx: &Context) {
    let gone = ctx.data_mut(|data| {
        let open = data.get_temp_mut_or_default::<Open>(open_id());
        let mut gone = Vec::new();
        open.0.retain_mut(|entry| {
            if entry.shown {
                entry.shown = false;
                true
            } else {
                gone.extend(entry.opener);
                false
            }
        });

        gone
    });

    for opener in gone {
        ctx.memory_mut(|memory| memory.request_focus(opener));
    }
}
