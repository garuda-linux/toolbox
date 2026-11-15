//! The desktop look a light test window starts with. Dark is the app's own default.

use egui::Theme;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use toolbox_gui::desktop::DesktopLook;
use toolbox_gui_kit::Palette;

/// The desktop look of a light window, as the desktop would send it. The
/// app takes a look only while its sender lives, so the sender waits on
/// `runtime` as long as the test does. Dark is the app's own default, so
/// it needs no look.
pub fn look_for(runtime: &Runtime, theme: Theme) -> Option<watch::Receiver<DesktopLook>> {
    if theme == Theme::Dark {
        return None;
    }

    let (sender, receiver) = watch::channel(DesktopLook::default());
    let mut look = DesktopLook::default();
    look.look.theme = theme;
    look.look.light = Palette::fallback(Theme::Light);
    look.look.dark = Palette::fallback(Theme::Dark);
    sender.send(look).ok()?;
    runtime.spawn(async move {
        let _keep_the_look = sender;
        std::future::pending::<()>().await;
    });
    Some(receiver)
}
