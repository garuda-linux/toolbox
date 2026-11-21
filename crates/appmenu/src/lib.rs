//! The window's global menu, as Plasma shows it in the title bar or a
//! panel. The app hands over a [`Menu`] whenever its state changes, this
//! crate serves it as `com.canonical.dbusmenu` on the session bus, and a
//! click comes back as the item's action. Egui free.

#[cfg(any(test, feature = "client"))]
pub mod client;
mod layout;
mod menu;
mod server;
#[cfg(test)]
mod testing;
mod window;
mod wire;

use std::num::TryFromIntError;

pub use self::menu::{Entry, Item, Menu, Modifier, Shortcut, Submenu, Toggle};
pub use self::server::{Exported, PATH};
pub use self::window::{Started, start};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the menu has more items than dbusmenu can number")]
    TooLarge(#[from] TryFromIntError),
    #[error("the session bus refused the menu")]
    Bus(#[from] zbus::Error),
    #[error("the session bus gave the menu no name")]
    NoName,
    #[error("the window has no handle")]
    Handle(#[from] raw_window_handle::HandleError),
    #[error("the X11 window id does not fit 32 bits")]
    XWindow(#[source] TryFromIntError),
    #[error("the window's Wayland surface is not one")]
    Surface(#[from] wayland_client::backend::InvalidId),
    #[error("the compositor's globals could not be read")]
    Globals(#[from] wayland_client::globals::GlobalError),
    #[error("the compositor's appmenu could not be bound")]
    Bind(#[from] wayland_client::globals::BindError),
    #[error("the compositor did not answer")]
    Dispatch(#[from] wayland_client::DispatchError),
}
