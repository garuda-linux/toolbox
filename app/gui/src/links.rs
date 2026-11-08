//! Opens web links in the desktop's browser and says so when one does not
//! open. egui's own `open_url` reports no failure, so a click that did
//! nothing would go unnoticed.
//!
//! `xdg-open` runs on a thread of its own: it may start the browser itself
//! and wait for it to close, so it gets no deadline that would kill it.

#[cfg(test)]
mod tests;

use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::{Context, Id};
use toolbox_exec::{Environment, Policy, capture};
use toolbox_gui_kit::toasts::{ToastKind, raise};
use toolbox_system::file_url;

use crate::i18n;

/// What `xdg-open` may print, kept for the log.
const OUTPUT_LIMIT: usize = 4096;

pub trait Opener: Send + Sync {
    fn open(&self, url: &str) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct XdgOpen;

impl Opener for XdgOpen {
    fn open(&self, url: &str) -> bool {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        let never = Arc::new(AtomicBool::new(false));
        let policy = Policy::tail(OUTPUT_LIMIT);

        match capture(&mut command, Environment::Inherited, &[], &policy, &never) {
            Ok(done) if done.code == Some(0) => true,
            Ok(done) => {
                tracing::warn!(code = ?done.code, errors = %done.errors.trim(), "xdg-open failed");
                false
            }
            Err(error) => {
                tracing::warn!("xdg-open did not run: {error:?}");
                false
            }
        }
    }
}

pub fn install(ctx: &Context, opener: Arc<dyn Opener>) {
    ctx.data_mut(|data| data.insert_temp(opener_id(), opener));
}

/// Anything else is refused: it could be read as an option of the opener.
pub fn open(ctx: &Context, url: &str) {
    let web = url.starts_with("https://") || url.starts_with("http://");
    if !web {
        tracing::warn!("not a web link, not opened: {url}");
        failed(ctx);
        return;
    }

    launch(ctx, url.to_owned());
}

pub fn open_file(ctx: &Context, path: &Path) {
    launch(ctx, file_url(path));
}

fn launch(ctx: &Context, url: String) {
    let opener = ctx
        .data(|data| data.get_temp::<Arc<dyn Opener>>(opener_id()))
        .unwrap_or_else(|| Arc::new(XdgOpen));

    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("open link".to_owned())
        .spawn(move || {
            if !opener.open(&url) {
                failed(&ctx);
            }
        });

    if let Err(error) = spawned {
        tracing::warn!("no thread to open a link: {error}");
    }
}

fn failed(ctx: &Context) {
    raise(ctx, ToastKind::Error, i18n::text(ctx, "toast-link-failed"));
    ctx.request_repaint();
}

fn opener_id() -> Id {
    Id::unique("toolbox_gui::links::opener")
}
