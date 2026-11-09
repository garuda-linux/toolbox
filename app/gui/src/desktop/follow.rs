//! The task that keeps the app's look in step with the desktop: it reads
//! the look at the start and again after each change of the portal's
//! appearance settings or of a look file.

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Duration;

use egui::Context;
use futures_lite::StreamExt as _;
use notify::{RecommendedWatcher, RecursiveMode, Watcher as _};
use tokio::runtime::Handle;
use tokio::sync::{mpsc, watch};
use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::desktop::family::Family;
use toolbox_gui_kit::desktop::look::{Look, Sources};
use toolbox_gui_kit::fonts::{FontFile, Weight};
use toolbox_gui_kit::icons::ThemeIcons;

use super::files::{DesktopFiles, Read, is_look_file};
use super::font_files::FontFinder;
use super::icon_files::{AppIcons, SystemIcons, app_icons, theme_icons};
use super::{DesktopLook, portal};
use crate::spawn::spawn_logged;

/// A theme switch writes several files in a row. One read after they
/// settle is enough.
const SETTLE: Duration = Duration::from_millis(200);

struct FilesRead {
    files: Read,
    ui_fonts: Vec<FontFile>,
    fixed_font: Option<FontFile>,
    emoji_font: Option<Arc<FontFile>>,
    kept: Kept,
}

/// What one read hands to the next, so the slow parts run only when
/// needed.
#[derive(Default)]
struct Kept {
    /// The system's fonts, scanned once.
    finder: Option<FontFinder>,
    /// The colour emoji font, looked for once.
    emoji: Option<Arc<FontFile>>,
    emoji_looked: bool,
    /// The icon theme's files, read again only when its name changes.
    icons: Option<Arc<ThemeIcons>>,
    /// The theme's app pictures, looked up again with the glyphs.
    apps: Option<Arc<AppIcons>>,
}

/// Starts following the desktop. The receiver holds the newest look, the
/// fallback until the first read lands.
pub fn spawn(
    runtime: &Handle,
    ctx: Context,
    family: Family,
    files: DesktopFiles,
    app_names: Arc<[String]>,
) -> watch::Receiver<DesktopLook> {
    let (sender, receiver) = watch::channel(DesktopLook::default());
    spawn_logged(
        runtime,
        "desktop look",
        follow(sender, ctx, family, files, app_names),
    );
    receiver
}

async fn follow(
    sender: watch::Sender<DesktopLook>,
    ctx: Context,
    family: Family,
    files: DesktopFiles,
    app_names: Arc<[String]>,
) {
    let settings = portal::connect().await;
    let mut portal_changes = portal::changes(settings.as_ref()).await;
    let (changed, mut file_changes) = mpsc::channel::<()>(1);

    // Kept alive for as long as the task runs.
    let _watcher = watch_files(&files, family, changed);
    let mut kept = Kept::default();

    loop {
        let Ok(read) = tokio::task::spawn_blocking({
            let files = files.clone();
            move || read_files(&files, family, kept)
        })
        .await
        else {
            return;
        };

        kept = read.kept;
        let sources = Sources {
            portal: portal::read(settings.as_ref()).await,
            desktop: read.files.desktop,
            prefer_dark: read.files.prefer_dark,
        };

        let look = DesktopLook {
            look: Look::resolve(&sources),
            ui_fonts: read.ui_fonts,
            fixed_font: read.fixed_font,
            emoji_font: read.emoji_font,
            icons: kept.icons.clone(),
            app_icons: kept.apps.clone(),
        };

        // An error means the app is gone.
        if sender.send(look).is_err() {
            return;
        }

        ctx.request_repaint();

        if kept.apps.is_none()
            && let Some(theme) = kept.icons.as_ref().map(|icons| icons.theme().to_owned())
        {
            let names = Arc::clone(&app_names);
            let found =
                tokio::task::spawn_blocking(move || app_icons(&theme, &names, &SystemIcons::APPS))
                    .await;

            if let Ok(found) = found {
                kept.apps = Some(Arc::new(found));
                sender.send_modify(|newest| newest.app_icons.clone_from(&kept.apps));
                ctx.request_repaint();
            }
        }

        tokio::select! {
            Some(()) = portal_changes.next() => {}
            Some(()) = file_changes.recv() => {}
            else => return,
        }

        tokio::time::sleep(SETTLE).await;
        while file_changes.try_recv().is_ok() {}
    }
}

/// The blocking half of a read: the look files, and the font and icon
/// files they name.
fn read_files(files: &DesktopFiles, family: Family, kept: Kept) -> FilesRead {
    let read = files.read(family);
    let desktop = read.desktop.as_ref();
    let specs = desktop.map_or((None, None), |desktop| {
        (desktop.font.clone(), desktop.fixed_font.clone())
    });

    // Always scanned: the emoji font is wanted whatever the desktop names.
    let finder = kept.finder.unwrap_or_else(FontFinder::system);
    let emoji_font = if kept.emoji_looked {
        kept.emoji
    } else {
        finder.emoji().map(Arc::new)
    };

    let icons = desktop
        .and_then(|desktop| desktop.icon_theme.as_deref())
        .map(|theme| {
            kept.icons
                .filter(|icons| icons.theme() == theme)
                .unwrap_or_else(|| Arc::new(theme_icons(theme, &SystemIcons::GLYPHS)))
        });

    // Slow (seconds for a theme without most of them), so they follow the
    // look in a second send instead of holding it up.
    let apps = desktop
        .and_then(|desktop| desktop.icon_theme.as_deref())
        .and_then(|theme| kept.apps.filter(|apps| apps.theme() == theme));

    let find = |spec: Option<FontSpec>| finder.find(&spec?);
    FilesRead {
        ui_fonts: specs
            .0
            .map(|spec| weights(&spec, |wanted| find(Some(wanted))))
            .unwrap_or_default(),
        fixed_font: find(specs.1),
        emoji_font: emoji_font.clone(),
        files: read,
        kept: Kept {
            finder: Some(finder),
            emoji: emoji_font,
            emoji_looked: true,
            icons,
            apps,
        },
    }
}

/// Watches the look files' folders. `None` when the system has no file
/// watching to offer, then only the portal is followed.
fn watch_files(
    files: &DesktopFiles,
    family: Family,
    changed: mpsc::Sender<()>,
) -> Option<RecommendedWatcher> {
    let handler = move |event: notify::Result<notify::Event>| {
        // Reads are skipped, or the task's own read would start the next.
        if let Ok(event) = event
            && !event.kind.is_access()
            && event.paths.iter().any(|path| is_look_file(path))
        {
            // A full channel means a read is due anyway.
            changed.try_send(()).ok();
        }
    };

    let mut watcher = match notify::recommended_watcher(handler) {
        Ok(watcher) => watcher,
        Err(error) => {
            tracing::info!("cannot watch the desktop's look files: {error}");
            return None;
        }
    };

    for folder in files.watched_dirs(family) {
        if let Err(error) = watcher.watch(&folder, RecursiveMode::NonRecursive) {
            tracing::info!("not watching {}: {error}", folder.display());
        }
    }

    Some(watcher)
}

/// The UI font at each weight the app draws: the desktop's own weight for
/// regular text, the others as the font has them.
fn weights(spec: &FontSpec, find: impl Fn(FontSpec) -> Option<FontFile>) -> Vec<FontFile> {
    Weight::ALL
        .into_iter()
        .filter_map(|weight| {
            let wanted = match weight {
                Weight::Regular => spec.weight,
                Weight::Medium | Weight::SemiBold | Weight::Bold => weight.css(),
            };

            find(FontSpec {
                weight: wanted,
                ..spec.clone()
            })
        })
        .collect()
}
