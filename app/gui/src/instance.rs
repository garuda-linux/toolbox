//! The bus calls [`Arrivals`] on its own threads. A Wayland token is used
//! at once from there, since a hidden window draws no frame to wait for.
//! The rest goes over the command bus and wakes the frame.

mod claim;
mod queue;
#[cfg(test)]
mod tests;

use std::sync::{Arc, OnceLock};

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle,
    WaylandWindowHandle,
};
use toolbox_core::features::diagnostics::Selection;
use toolbox_instance::activation::{Activation, Token};
use toolbox_instance::bus::queue::Ask;
use toolbox_instance::raise::Raiser;
pub use toolbox_services::instance::PageAction;

pub use self::claim::{Single, claim};
use crate::command::{AppCommand, CommandSender};
use crate::page::PageId;

/// Brings the window to the front with a token, a port so the tests need
/// no compositor.
pub trait Raise: Send + Sync {
    fn raise(&self, token: &Token);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrival {
    pub page: Option<PageId>,
    pub action: Option<PageAction>,
    /// The window was asked to the front with a token already. Otherwise
    /// the app asks for focus and attention itself.
    pub raised: bool,
}

pub struct Arrivals {
    commands: CommandSender,
    ctx: OnceLock<egui::Context>,
    /// Taken on exit, before the toolkit closes its display: dropping the
    /// guest connection later would touch freed memory.
    raise: Mutex<Option<Arc<dyn Raise>>>,
    /// Questions about the queue from commands, answered by the app's
    /// logic, which owns the queue.
    asks: (Sender<Ask>, Receiver<Ask>),
}

impl Raise for Raiser {
    fn raise(&self, token: &Token) {
        self.activate(token);
    }
}

impl std::fmt::Debug for Arrivals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arrivals")
            .field("raise", &self.raise.lock().is_some())
            .finish_non_exhaustive()
    }
}

impl Arrival {
    /// `activation` with its page and action checked against this app.
    /// What it does not know is dropped, the window still comes up.
    pub fn of(activation: &Activation, raised: bool) -> Self {
        let page = activation
            .page
            .as_ref()
            .and_then(|name| PageId::from_slug(name.as_ref()));

        if activation.page.is_some() && page.is_none() {
            tracing::warn!("another start asked for a page this app does not have");
        }

        let action = activation
            .action
            .as_ref()
            .filter(|_| page == Some(PageId::Diagnostics))
            .and_then(|name| Selection::from_id(name.as_ref()))
            .map(PageAction::Collect);

        Self {
            page,
            action,
            raised,
        }
    }
}

impl Arrivals {
    #[must_use = "the arrivals do nothing until the bus calls them"]
    pub fn new(commands: CommandSender) -> Self {
        Self {
            commands,
            ctx: OnceLock::new(),
            raise: Mutex::new(None),
            asks: crossbeam_channel::unbounded(),
        }
    }

    /// The window's context, which an arrival wakes. Arrivals before it
    /// wait in the command bus.
    pub fn set_context(&self, ctx: egui::Context) {
        if self.ctx.set(ctx).is_err() {
            tracing::debug!("the arrivals had a context already");
        }
    }

    pub fn set_raise(&self, raise: Arc<dyn Raise>) {
        *self.raise.lock() = Some(raise);
    }

    /// The window closes: no raise from now on. Called before the toolkit
    /// closes its display, and a raise that runs holds the lock until it
    /// is done.
    pub fn close(&self) {
        *self.raise.lock() = None;
    }
}

impl toolbox_instance::bus::Arrivals for Arrivals {
    fn arrive(&self, activation: Activation) {
        // The lock is held through the raise, so `close` waits for it.
        let raised = activation.token.as_ref().is_some_and(|token| {
            self.raise
                .lock()
                .as_ref()
                .map(|raiser| raiser.raise(token))
                .is_some()
        });

        let arrival = Arrival::of(&activation, raised);
        tracing::info!("another start asked for {:?}", arrival.page);
        self.commands.send(AppCommand::Activated(arrival));

        if let Some(ctx) = self.ctx.get() {
            ctx.request_repaint();
        }
    }
}

pub fn ready(
    arrivals: &Arrivals,
    ctx: &egui::Context,
    window: &impl HasWindowHandle,
    display: &impl HasDisplayHandle,
) {
    arrivals.set_context(ctx.clone());

    let Some((surface, wayland)) = wayland_handles(window, display) else {
        return;
    };

    #[expect(
        unsafe_code,
        reason = "joins winit's wl_display with the window's wl_surface, see the SAFETY comment"
    )]
    // SAFETY: winit's display and the main window's surface live as long
    // as the window. `Arrivals::close` drops the raiser before the window
    // goes, and a raise that runs holds its lock until it is done.
    let Some(raiser) = (unsafe { Raiser::open(wayland.display, surface.surface) }) else {
        return;
    };

    arrivals.set_raise(Arc::new(raiser));
}

/// The window's Wayland surface and display. `None` on any other window
/// system, or while a handle is not there.
fn wayland_handles(
    window: &impl HasWindowHandle,
    display: &impl HasDisplayHandle,
) -> Option<(WaylandWindowHandle, WaylandDisplayHandle)> {
    let window = window.window_handle().ok()?.as_raw();
    let display = display.display_handle().ok()?.as_raw();
    match (window, display) {
        (RawWindowHandle::Wayland(surface), RawDisplayHandle::Wayland(wayland)) => {
            Some((surface, wayland))
        }
        _ => None,
    }
}
