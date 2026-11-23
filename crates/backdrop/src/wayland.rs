//! The window's surface asks for blur through the protocol the compositor
//! offers. The probe reads the globals on winit's display, which is the one
//! the surface lives on, so the objects must come from that connection.

use raw_window_handle::{WaylandDisplayHandle, WaylandWindowHandle};
use wayland_client::backend::{Backend, ObjectId};
use wayland_client::globals::{GlobalList, GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_region::WlRegion;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, Proxy as _, QueueHandle, WEnum, delegate_noop};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_manager_v1::{
    self, ExtBackgroundEffectManagerV1,
};
use wayland_protocols::ext::background_effect::v1::client::ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1;
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur::OrgKdeKwinBlur;
use wayland_protocols_plasma::blur::client::org_kde_kwin_blur_manager::OrgKdeKwinBlurManager;

use crate::Error;
use crate::choice::{Path, decide, wayland_offers};

const EXTENSION: &str = "ext_background_effect_manager_v1";

/// Larger than any window. The compositor clips the region to the surface.
const WHOLE: i32 = 0xFFFF;

/// The events the probe reads. The effect objects send none that matter
/// after the capabilities.
#[derive(Default)]
struct Probe {
    capabilities: Option<u32>,
}

delegate_noop!(Probe: ignore WlCompositor);
delegate_noop!(Probe: ignore WlRegion);
delegate_noop!(Probe: ignore ExtBackgroundEffectSurfaceV1);
delegate_noop!(Probe: ignore OrgKdeKwinBlurManager);
delegate_noop!(Probe: ignore OrgKdeKwinBlur);

impl Dispatch<WlRegistry, GlobalListContents> for Probe {
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

impl Dispatch<ExtBackgroundEffectManagerV1, ()> for Probe {
    fn event(
        state: &mut Self,
        _: &ExtBackgroundEffectManagerV1,
        event: ext_background_effect_manager_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_background_effect_manager_v1::Event::Capabilities { flags } = event {
            state.capabilities = Some(match flags {
                WEnum::Value(flags) => flags.bits(),
                WEnum::Unknown(bits) => bits,
            });
        }
    }
}

/// Asks the compositor for the path it offers, and applies it to `window`'s
/// surface.
pub fn apply(display: WaylandDisplayHandle, window: WaylandWindowHandle) -> Result<Path, Error> {
    let connection = connect(display);
    let surface = surface(&connection, window)?;
    let (globals, mut queue) = registry_queue_init::<Probe>(&connection)?;
    let handle = queue.handle();
    let mut probe = Probe::default();

    let names: Vec<String> = globals
        .contents()
        .with_list(|list| list.iter().map(|global| global.interface.clone()).collect());

    // The capabilities arrive with the bind. The manager is kept only when
    // the path takes it.
    let extension = if names.iter().any(|name| name == EXTENSION) {
        let manager: ExtBackgroundEffectManagerV1 = globals.bind(&handle, 1..=1, ())?;
        queue.roundtrip(&mut probe)?;
        Some(manager)
    } else {
        None
    };

    let offers = wayland_offers(&names, probe.capabilities);
    let path = decide(offers);
    tracing::debug!(?offers, capabilities = ?probe.capabilities, "desktop blur offers");

    take(path, extension, &globals, &handle, &surface)?;

    // The next commit of the window applies the region. The objects live as
    // long as the surface, and the connection is winit's, which may be gone
    // before a destructor could run, so nothing is dropped here.
    queue.roundtrip(&mut probe)?;
    std::mem::forget((connection, queue));
    Ok(path)
}

/// Sends the requests of `path`. Its objects are kept for the window's life.
fn take(
    path: Path,
    extension: Option<ExtBackgroundEffectManagerV1>,
    globals: &GlobalList,
    handle: &QueueHandle<Probe>,
    surface: &WlSurface,
) -> Result<(), Error> {
    match path {
        Path::Extension => {
            // The bind is a fallback the probe never needs, since only a
            // bound manager confirms the extension, and it fails loudly when
            // the global is gone.
            let manager = match extension {
                Some(manager) => manager,
                None => globals.bind(handle, 1..=1, ())?,
            };

            let compositor: WlCompositor = globals.bind(handle, 1..=6, ())?;
            let region = whole_region(&compositor, handle);
            let effect = manager.get_background_effect(surface, handle, ());
            effect.set_blur_region(Some(&region));
            region.destroy();

            std::mem::forget((manager, effect));
        }
        Path::KdeBlur => {
            let compositor: WlCompositor = globals.bind(handle, 1..=6, ())?;
            let manager: OrgKdeKwinBlurManager = globals.bind(handle, 1..=1, ())?;
            let region = whole_region(&compositor, handle);
            let blur = manager.create(surface, handle, ());
            blur.set_region(Some(&region));
            blur.commit();
            region.destroy();

            std::mem::forget((manager, blur));
        }
        // A Wayland compositor never offers the X11 property.
        Path::X11Kwin | Path::Opaque => {}
    }

    Ok(())
}

/// The connection on winit's display. Requests on it go out on winit's socket.
fn connect(display: WaylandDisplayHandle) -> Connection {
    #[expect(
        unsafe_code,
        reason = "winit hands its display over as a libwayland pointer"
    )]
    // SAFETY: the display comes from the handle of a window the caller
    // borrows until the probe returns, so it stays valid that long. Nothing
    // here closes it, and the backend is only forgotten by `apply`.
    let backend = unsafe { Backend::from_foreign_display(display.display.as_ptr().cast()) };
    Connection::from_backend(backend)
}

/// The window's surface on `connection`, by the id winit made it with.
fn surface(connection: &Connection, window: WaylandWindowHandle) -> Result<WlSurface, Error> {
    #[expect(
        unsafe_code,
        reason = "winit hands its surface over as a libwayland pointer"
    )]
    // SAFETY: the surface comes from the same borrowed window handle and is
    // only used before the probe returns.
    let id = unsafe { ObjectId::from_ptr(WlSurface::interface(), window.surface.as_ptr().cast()) };
    Ok(WlSurface::from_id(connection, id?)?)
}

fn whole_region(compositor: &WlCompositor, handle: &QueueHandle<Probe>) -> WlRegion {
    let region = compositor.create_region(handle, ());
    region.add(0, 0, WHOLE, WHOLE);
    region
}
