//! KWin's `org_kde_kwin_appmenu`: the window's surface names the bus
//! address of its menu.

use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};
use wayland_client::backend::{Backend, ObjectId};
use wayland_client::globals::{BindError, GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, Proxy as _, QueueHandle, delegate_noop};
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu::OrgKdeKwinAppmenu;
use wayland_protocols_plasma::appmenu::client::org_kde_kwin_appmenu_manager::OrgKdeKwinAppmenuManager;

use crate::Error;
use crate::server::PATH;

/// Neither appmenu object sends events.
struct Events;

delegate_noop!(Events: ignore OrgKdeKwinAppmenuManager);
delegate_noop!(Events: ignore OrgKdeKwinAppmenu);

impl Dispatch<WlRegistry, GlobalListContents> for Events {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// Names `service` as the menu of the window's surface. `false` when the
/// compositor has no global menu.
pub(super) fn bind(
    display: WaylandDisplayHandle,
    window: WaylandWindowHandle,
    service: &str,
) -> Result<bool, Error> {
    #[expect(
        unsafe_code,
        reason = "winit hands its display over as a libwayland pointer"
    )]
    // SAFETY: the display comes from the handle of a window the caller
    // borrows until this returns, so it stays valid that long. The backend
    // is dropped before that, or never (see the end).
    let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };

    #[expect(
        unsafe_code,
        reason = "winit hands its surface over as a libwayland pointer"
    )]
    // SAFETY: the surface comes from the same borrowed window handle and
    // is only used before this returns.
    let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), window.surface.as_ptr().cast()) };

    let connection = Connection::from_backend(backend);
    let surface = WlSurface::from_id(&connection, id?)?;
    let (globals, mut queue) = registry_queue_init::<Events>(&connection)?;
    let handle = queue.handle();
    let manager: OrgKdeKwinAppmenuManager = match globals.bind(&handle, 1..=2, ()) {
        Ok(manager) => manager,
        Err(BindError::NotPresent) => return Ok(false),
        Err(error) => return Err(error.into()),
    };

    let menu = manager.create(&surface, &handle, ());
    menu.set_address(service.to_owned(), PATH.to_owned());
    queue.roundtrip(&mut Events)?;

    // Dropping the backend destroys its queue on winit's display, which
    // winit may have disconnected by then. The menu lives as long as the
    // window, so it is kept for good.
    std::mem::forget((connection, queue, manager, menu));
    Ok(true)
}
