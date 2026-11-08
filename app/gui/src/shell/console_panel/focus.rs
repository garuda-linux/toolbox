//! Where the keyboard goes once Apply opened the Console. The pill that
//! held the focus is gone then, and the next Tab would start over at the
//! sidebar.

use egui::{Context, Id, Response};

fn mark() -> Id {
    Id::unique("toolbox_gui::console::focus_on_open")
}

pub fn on_open(ctx: &Context) {
    ctx.data_mut(|data| data.insert_temp(mark(), true));
}

pub fn take(ctx: &Context, target: &Response) {
    if ctx
        .data_mut(|data| data.remove_temp::<bool>(mark()))
        .is_some()
    {
        target.request_focus();
    }
}

fn held_mark() -> Id {
    Id::unique("toolbox_gui::console::header_focus")
}

/// Keeps the keyboard in the header as the run ends. Cancel goes then, and
/// the header's controls change, which drops the focus. Close, the one
/// control that is always there, takes it back rather than the sidebar on
/// the next Tab. Only at that moment: a focus the user moves or clears
/// later stays as they left it. `cancel` is the Cancel drawn this frame.
pub fn keep_in_header(ctx: &Context, running: bool, cancel: Option<&Response>, close: &Response) {
    let drawn: Vec<&Response> = cancel.into_iter().chain([close]).collect();
    let held = drawn
        .iter()
        .find(|control| control.has_focus())
        .map(|control| control.id);

    let before = ctx.data_mut(|data| {
        let before = data.get_temp::<Option<Id>>(held_mark()).flatten();
        data.insert_temp(held_mark(), held.filter(|_| running));
        before
    });

    let Some(before) = before else {
        return;
    };

    // Egui still names a control that is gone as focused in that frame.
    let focused = ctx.memory(egui::Memory::focused);
    let still_there = drawn.iter().any(|control| control.id == before);
    if !running && (focused.is_none() || (focused == Some(before) && !still_there)) {
        close.request_focus();
    }
}
