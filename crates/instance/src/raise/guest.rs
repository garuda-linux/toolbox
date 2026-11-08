//! A guest on the toolkit's Wayland connection: an own event queue beside
//! the toolkit's, which reads the socket for both.

use std::ffi::c_void;
use std::ptr::NonNull;

use wayland_client::backend::{Backend, ObjectId};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Proxy as _};

/// Joins the toolkit's connection.
///
/// # Safety
///
/// `display` must be a live `wl_display` that outlives the connection.
#[expect(
    unsafe_code,
    reason = "wraps the wl_display pointer the toolkit handed out, see the SAFETY comment"
)]
pub unsafe fn join(display: NonNull<c_void>) -> Connection {
    // SAFETY: the caller passes a live wl_display and keeps it alive for
    // the connection's lifetime. A foreign display is never closed by the
    // backend.
    let backend = unsafe { Backend::from_foreign_display(display.as_ptr().cast()) };
    Connection::from_backend(backend)
}

/// The toolkit's `surface` as a proxy on `connection`.
///
/// # Safety
///
/// `surface` must be a live `wl_surface` of `connection`'s display.
#[expect(
    unsafe_code,
    reason = "wraps the wl_surface pointer the toolkit handed out, see the SAFETY comment"
)]
pub unsafe fn surface(connection: &Connection, surface: NonNull<c_void>) -> Option<WlSurface> {
    // SAFETY: the caller passes a live wl_surface proxy of this display.
    let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), surface.as_ptr().cast()) };
    WlSurface::from_id(connection, id.ok()?).ok()
}
