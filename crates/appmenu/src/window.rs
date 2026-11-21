//! Ties the served menu to the window, so the desktop shows it as that
//! window's menu: an X11 window through the Registrar on the session
//! bus, a Wayland surface through KWin's appmenu protocol.

#[cfg(test)]
mod tests;
mod wayland;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use zbus::proxy::{Builder, CacheProperties};
use zbus::{Connection, ObjectPath, Proxy, fdo};

use crate::Error;
use crate::menu::Menu;
use crate::server::{Exported, PATH};

const REGISTRAR: &str = "com.canonical.AppMenu.Registrar";
const REGISTRAR_PATH: &str = "/com/canonical/AppMenu/Registrar";

/// The served menu, and whether the desktop shows it for the window.
pub struct Started<A> {
    pub exported: Exported<A>,
    pub bound: bool,
}

/// Serves `menu` as the global menu of the window behind `handles`.
///
/// The menu gets its own session-bus connection and stays served where the
/// window system or the desktop shows no global menu, as under a headless
/// compositor. `clicks` runs on the bus's threads.
///
/// # Errors
///
/// When the window has no handle, the session bus is out of reach or
/// refuses the menu, or the compositor fails the binding.
#[expect(
    clippy::future_not_send,
    reason = "the window's handles hold only on the thread that borrows them, which blocks on this"
)]
pub async fn start<A, H>(
    handles: &H,
    menu: &Menu<A>,
    clicks: impl Fn(A) + Send + Sync + 'static,
) -> Result<Started<A>, Error>
where
    A: Clone + Send + Sync + 'static,
    H: HasWindowHandle + HasDisplayHandle,
{
    let window = handles.window_handle()?.as_raw();
    let display = handles.display_handle()?.as_raw();
    let exported = Exported::serve(Connection::session().await?, menu, clicks).await?;
    let connection = exported.connection();
    let bound = match (window, display) {
        (RawWindowHandle::Xlib(x11), _) => {
            register(
                connection,
                u32::try_from(x11.window).map_err(Error::XWindow)?,
            )
            .await?
        }
        (RawWindowHandle::Xcb(x11), _) => register(connection, x11.window.get()).await?,
        (RawWindowHandle::Wayland(surface), RawDisplayHandle::Wayland(wayland)) => {
            let service = connection.unique_name().ok_or(Error::NoName)?;
            wayland::bind(wayland, surface, service.as_str())?
        }
        _ => false,
    };

    Ok(Started { exported, bound })
}

/// `false` when no Registrar runs, as outside Plasma.
async fn register(connection: &Connection, window: u32) -> Result<bool, Error> {
    let registrar: Proxy<'_> = Builder::new(connection)
        .destination(REGISTRAR)
        .path(REGISTRAR_PATH)
        .interface(REGISTRAR)
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let path = ObjectPath::from_static_str_unchecked(PATH);
    match registrar
        .call::<_, _, ()>("RegisterWindow", &(window, path))
        .await
    {
        Ok(()) => Ok(true),
        Err(error) => {
            let error = fdo::Error::from(error);
            if matches!(error, fdo::Error::ServiceUnknown(_)) {
                Ok(false)
            } else {
                Err(Error::Bus(error.into()))
            }
        }
    }
}
