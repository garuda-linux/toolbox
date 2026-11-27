//! The review with real git in temp folders. The stand-in for paru copies a
//! repository into the clone, as `paru -G` leaves one, so the diffs are git's.

mod ask;
mod fetch;
mod hostile;
mod pair;
mod settings;
mod show;
mod support;
