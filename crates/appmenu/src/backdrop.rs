//! The blur behind the window. A compositor blurs what lies behind a
//! translucent surface once the window asks for it through a protocol the
//! compositor offers. Asked for at start, before the first frame.

mod choice;
#[cfg(test)]
mod tests;
mod wayland;
mod x11;

pub use choice::Path;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the Wayland surface has no valid id: {0}")]
    Surface(#[from] wayland_client::backend::InvalidId),
    #[error("cannot read the Wayland globals: {0}")]
    Globals(#[from] wayland_client::globals::GlobalError),
    #[error("cannot bind a Wayland global: {0}")]
    Bind(#[from] wayland_client::globals::BindError),
    #[error("the Wayland roundtrip failed: {0}")]
    Dispatch(#[from] wayland_client::DispatchError),
    #[error("cannot connect to the X server: {0}")]
    X11Connect(#[from] x11rb::errors::ConnectError),
    #[error("the X server refused a request: {0}")]
    X11Connection(#[from] x11rb::errors::ConnectionError),
    #[error("the X server did not answer: {0}")]
    X11Reply(#[from] x11rb::errors::ReplyError),
    #[error("the X server has no screen {0}")]
    NoScreen(usize),
    #[error("the X window id {0} does not fit 32 bits")]
    WindowId(u64),
}

/// Asks the compositor which blur it offers for `window`, and applies it.
///
/// The request is made whatever the user's setting, which only decides
/// whether the background is translucent. A failure gives [`Path::Opaque`]
/// and says why in the log.
#[must_use = "a blur path that is not applied leaves the window opaque"]
pub fn install(window: &(impl HasWindowHandle + HasDisplayHandle)) -> Path {
    let (Ok(window_handle), Ok(display_handle)) = (window.window_handle(), window.display_handle())
    else {
        tracing::warn!("desktop blur: the window has no handles, the window stays opaque");
        return Path::Opaque;
    };

    let asked = match (window_handle.as_raw(), display_handle.as_raw()) {
        (RawWindowHandle::Wayland(surface), RawDisplayHandle::Wayland(display)) => {
            wayland::apply(display, surface)
        }
        (RawWindowHandle::Xlib(xlib), RawDisplayHandle::Xlib(_)) => u32::try_from(xlib.window)
            .map_err(|_too_wide| Error::WindowId(xlib.window))
            .and_then(x11::apply),
        (RawWindowHandle::Xcb(xcb), RawDisplayHandle::Xcb(_)) => x11::apply(xcb.window.get()),
        _ => Ok(Path::Opaque),
    };

    match asked {
        Ok(path) => {
            tracing::info!("desktop blur: {path}");
            path
        }
        Err(error) => {
            tracing::warn!(
                "desktop blur: cannot ask the compositor, the window stays opaque: {error}"
            );
            Path::Opaque
        }
    }
}
