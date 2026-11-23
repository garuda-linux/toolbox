//! The desktop look a test window starts with. Dark is the app's own default
//! unless a guide picture asks for the user's desktop.

mod icons;

use std::cell::Cell;
use std::path::Path;

use egui::Theme;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use toolbox_catalog::ColourSchemeCatalog;
use toolbox_config::Colours;
use toolbox_gui::desktop::DesktopLook;
use toolbox_gui::schemes;
use toolbox_gui_kit::Palette;
use toolbox_gui_kit::desktop::kde;
use toolbox_gui_kit::desktop::look::{Look, Sources};
use toolbox_gui_kit::testing::desktop::kde::INSTALLED_KDEGLOBALS;
use toolbox_plan::SchemeId;

thread_local! {
    /// Per thread, so each test has its own flag and the snapshot tests keep
    /// the app's fallback look.
    static PICTURED: Cell<bool> = const { Cell::new(false) };
}

/// Renders `picture` with the user's desktop look, as the guide shows it.
pub fn pictured<T>(picture: impl FnOnce() -> T) -> T {
    PICTURED.with(|flag| flag.set(true));
    let rendered = picture();
    PICTURED.with(|flag| flag.set(false));
    rendered
}

/// The desktop look of a light window, as the desktop would send it. The
/// app takes a look only while its sender lives, so the sender waits on
/// `runtime` as long as the test does. Dark is the app's own default, so
/// it needs no look.
pub fn look_for(runtime: &Runtime, theme: Theme) -> Option<watch::Receiver<DesktopLook>> {
    if PICTURED.with(Cell::get) {
        return Some(send(runtime, pictured_look(theme)));
    }

    if theme == Theme::Dark {
        return None;
    }

    let mut look = DesktopLook::default();
    look.look.theme = theme;
    look.look.light = Palette::fallback(Theme::Light);
    look.look.dark = Palette::fallback(Theme::Dark);
    Some(send(runtime, look))
}

/// The user's desktop for a guide picture: the Mokka colours of the
/// captured `kdeglobals` in dark, and the app's Catppuccin Latte scheme as
/// the light palette. No Latte `kdeglobals` is captured, so the light
/// window is set by the theme the picture asks for. The icons are the
/// desktop's Tela theme of that theme, read from the vendored copy. Only
/// `pictured` reaches this, so the snapshot tests keep the bundled icons.
fn pictured_look(theme: Theme) -> DesktopLook {
    let desktop = kde::parse(INSTALLED_KDEGLOBALS);
    let mut look = Look::resolve(&Sources {
        desktop: Some(desktop),
        ..Sources::default()
    });

    look.theme = theme;

    let catalog = ColourSchemeCatalog::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog"),
    )
    .expect("the shipped colour schemes load");

    let latte = Colours {
        dark_scheme: None,
        light_scheme: Some(SchemeId::new("catppuccin-latte").expect("a valid scheme id")),
    };

    let (dark, light) = schemes::palettes((look.dark, look.light), &catalog, &latte);
    look.dark = dark;
    look.light = light;

    DesktopLook {
        look,
        icons: Some(icons::for_theme(theme)),
        ..DesktopLook::default()
    }
}

fn send(runtime: &Runtime, look: DesktopLook) -> watch::Receiver<DesktopLook> {
    let (sender, receiver) = watch::channel(DesktopLook::default());
    sender.send(look).expect("the receiver is alive");
    runtime.spawn(async move {
        let _keep_the_look = sender;
        std::future::pending::<()>().await;
    });
    receiver
}
