//! The bus calls [`Arrivals`] on its own threads. An arrival goes over the
//! command bus and wakes the frame.

mod claim;
mod queue;
#[cfg(test)]
mod tests;

use std::sync::OnceLock;

use crossbeam_channel::{Receiver, Sender};
use toolbox_core::features::diagnostics::Selection;
use toolbox_instance::activation::Activation;
use toolbox_instance::bus::queue::Ask;
pub use toolbox_services::instance::PageAction;

pub use self::claim::{Single, claim};
use crate::command::{AppCommand, CommandSender};
use crate::page::PageId;

/// Questions the app has not answered yet. Any process of the session can
/// call, and each edit may carry 256 KiB, so a flood is refused instead of
/// parked.
pub const WAITING_ASKS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrival {
    pub page: Option<PageId>,
    pub action: Option<PageAction>,
    /// The window was opened for this start and is in front already.
    /// Otherwise the app asks for focus and attention.
    pub raised: bool,
}

pub struct Arrivals {
    commands: CommandSender,
    ctx: OnceLock<egui::Context>,
    /// Questions about the queue from commands, answered by the app's
    /// logic, which owns the queue.
    asks: (Sender<Ask>, Receiver<Ask>),
}

impl std::fmt::Debug for Arrivals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arrivals").finish_non_exhaustive()
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

    /// The collection to start now. Any process of the session can send
    /// an arrival, and a collection may ask for a password, so a window
    /// that was not `in_front` only opens the page and waits for a click.
    pub const fn collection(&self, in_front: bool) -> Option<Selection> {
        match &self.action {
            Some(PageAction::Collect(selection)) if in_front => Some(*selection),
            Some(PageAction::Collect(_)) | None => None,
        }
    }
}

impl Arrivals {
    #[must_use = "the arrivals do nothing until the bus calls them"]
    pub fn new(commands: CommandSender) -> Self {
        Self {
            commands,
            ctx: OnceLock::new(),
            asks: crossbeam_channel::bounded(WAITING_ASKS),
        }
    }

    /// The window's context, which an arrival wakes. Arrivals before it
    /// wait in the command bus.
    pub fn set_context(&self, ctx: egui::Context) {
        if self.ctx.set(ctx).is_err() {
            tracing::debug!("the arrivals had a context already");
        }
    }
}

impl toolbox_instance::bus::Arrivals for Arrivals {
    fn arrive(&self, activation: Activation) {
        let arrival = Arrival::of(&activation, false);
        tracing::info!("another start asked for {:?}", arrival.page);
        self.commands.send(AppCommand::Activated(arrival));

        if let Some(ctx) = self.ctx.get() {
            ctx.request_repaint();
        }
    }
}
