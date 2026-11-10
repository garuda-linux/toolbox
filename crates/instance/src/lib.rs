//! One Garuda Toolbox per session. The first start owns its name on the
//! session bus and speaks `org.freedesktop.Application` there. A later
//! start hands its wish over and exits. The first one then asks for its
//! window to come to the front.
//!
//! `activation` holds the wish as plain values, `bus` the D-Bus side. Egui
//! free, so the app's tests drive it with fakes and never need a session
//! bus.

pub mod activation;
pub mod bus;
