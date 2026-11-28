//! KWin on X11 blurs behind a window that carries its property. The
//! property's value is a region list, and an empty one means the whole window.

use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use super::Error;
use super::choice::{Offers, Path, decide};

/// Blurs `window` when the window manager is KWin.
pub fn apply(window: u32) -> Result<Path, Error> {
    let (connection, screen) = RustConnection::connect(None)?;
    let root = connection
        .setup()
        .roots
        .get(screen)
        .ok_or(Error::NoScreen(screen))?
        .root;

    let kwin = is_kwin(&connection, root)?;

    let path = decide(Offers {
        x11_kwin: kwin,
        ..Offers::default()
    });

    if path == Path::X11Kwin {
        let property = atom(&connection, b"_KDE_NET_WM_BLUR_BEHIND_REGION")?;
        connection.change_property32(
            PropMode::REPLACE,
            window,
            property,
            AtomEnum::CARDINAL,
            &[],
        )?;
        connection.flush()?;
    }

    Ok(path)
}

/// KWin names itself in `_NET_WM_NAME` on the check window the root
/// window's `_NET_SUPPORTING_WM_CHECK` points to.
fn is_kwin(connection: &RustConnection, root: u32) -> Result<bool, Error> {
    let check = atom(connection, b"_NET_SUPPORTING_WM_CHECK")?;
    let name = atom(connection, b"_NET_WM_NAME")?;
    let utf8 = atom(connection, b"UTF8_STRING")?;

    let check_reply = connection
        .get_property(false, root, check, AtomEnum::WINDOW, 0, 1)?
        .reply()?;

    let Some(check_window) = check_reply.value32().and_then(|mut ids| ids.next()) else {
        return Ok(false);
    };

    let name_reply = connection
        .get_property(false, check_window, name, utf8, 0, 64)?
        .reply()?;

    Ok(name_reply.value == b"KWin")
}

fn atom(connection: &RustConnection, name: &[u8]) -> Result<u32, Error> {
    Ok(connection.intern_atom(false, name)?.reply()?.atom)
}
