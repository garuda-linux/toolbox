//! Brings the window to the front on Wayland with `xdg_activation_v1`.
//!
//! A later start hands over the token its launcher gave it. The window
//! toolkit has no way to use a token from another process on a window
//! that is open already, so this joins the toolkit's Wayland connection
//! as a [`guest`] and asks for the activation itself. The
//! compositor decides: with a valid token the window comes up, without
//! one it may only ask for attention.
//!
//! The request goes out from the bus's thread. A hidden window gets no
//! frames, so waiting for the next frame might wait for ever.

mod guest;
mod roundtrip;

use std::ffi::c_void;
use std::ptr::NonNull;

use parking_lot::Mutex;
use wayland_client::protocol::wl_callback::{self, WlCallback};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy as _, QueueHandle, delegate_noop};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;

use self::roundtrip::{Answered, within};
use crate::activation::Token;

#[derive(Default)]
struct Globals {
    activation: Option<XdgActivationV1>,
}

impl Dispatch<WlCallback, Answered> for Globals {
    fn event(
        _state: &mut Self,
        _proxy: &WlCallback,
        event: wl_callback::Event,
        data: &Answered,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        roundtrip::answer(data, &event);
    }
}

impl Dispatch<WlRegistry, ()> for Globals {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version: _,
        } = event
        else {
            return;
        };

        // Version 1: nothing newer is used.
        if interface == XdgActivationV1::interface().name && state.activation.is_none() {
            state.activation = Some(proxy.bind(name, 1, qhandle, ()));
        }
    }
}

delegate_noop!(Globals: ignore XdgActivationV1);

/// The guest connection with the window's surface. It must never be
/// dropped after the toolkit closed its display, so the app keeps it for
/// the life of the process.
pub struct Raiser {
    connection: Connection,
    activation: XdgActivationV1,
    surface: WlSurface,
    /// The registry and the activation object live on it. Its events are
    /// dispatched at each use, so they never pile up.
    queue: Mutex<EventQueue<Globals>>,
}

impl Raiser {
    /// Joins the toolkit's connection and binds `xdg_activation_v1`.
    /// `None` without the protocol. Blocks for one round trip.
    ///
    /// # Safety
    ///
    /// `display` must be a live `wl_display` that outlives the raiser,
    /// and `surface` a live `wl_surface` of it that outlives the raiser.
    #[expect(
        unsafe_code,
        reason = "forwards the wl_display and wl_surface pointers the toolkit handed out, see the SAFETY comments"
    )]
    pub unsafe fn open(display: NonNull<c_void>, surface: NonNull<c_void>) -> Option<Self> {
        // SAFETY: the caller passes a live wl_display that outlives the
        // raiser.
        let connection = unsafe { guest::join(display) };
        let mut queue = connection.new_event_queue();
        let mut globals = Globals::default();

        connection.display().get_registry(&queue.handle(), ());
        if let Err(error) = within(&connection, &mut queue, &mut globals) {
            tracing::warn!("raise: no answer from the compositor: {error}");
            return None;
        }

        let Some(activation) = globals.activation else {
            tracing::debug!("raise: the compositor does not offer xdg_activation_v1");
            return None;
        };

        // SAFETY: the caller passes a live wl_surface of this display.
        let surface = unsafe { guest::surface(&connection, surface) }?;

        Some(Self {
            connection,
            activation,
            surface,
            queue: Mutex::new(queue),
        })
    }

    pub fn activate(&self, token: &Token) {
        let drained = self.queue.lock().dispatch_pending(&mut Globals::default());
        if let Err(error) = drained {
            tracing::debug!("raise: cannot dispatch Wayland events: {error}");
        }

        self.activation
            .activate(token.as_str().to_owned(), &self.surface);

        if let Err(error) = self.connection.flush() {
            tracing::debug!("raise: cannot flush the Wayland connection: {error}");
        }

        tracing::debug!("raise: asked the compositor to activate the window");
    }
}
