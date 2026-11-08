//! One Garuda Toolbox per session. The first start owns its name on the
//! session bus and speaks `org.freedesktop.Application` there. A later
//! start hands its wish and its activation token over and exits. The
//! first one then brings its window to the front.
//!
//! `activation` holds the wish as plain values, `bus` the D-Bus side,
//! `raise` the Wayland side. Egui free, so the app's tests drive it with
//! fakes and never need a session bus.

pub mod activation;
pub mod bus;
pub mod raise;
