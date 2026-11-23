//! The glass look: with the desktop blur on, the page area paints no opaque
//! window fill over the translucent layer, and with it off the fill stays.

use egui::{Context, Pos2, Theme};
use toolbox_gui::{PreferencesSetup, Setup};
use toolbox_gui_kit::testing::opaque_fills_at;
use toolbox_gui_kit::tokens;

use crate::common::{HEIGHT, launch_with, shared_runtime, window};
use crate::look::look_for;

/// Inside the page area, below the header and clear of the sidebar.
const IN_THE_PAGE: Pos2 = Pos2::new(560.0, 360.0);

fn glass_on(setup: Setup, ctx: &Context) -> Setup {
    toolbox_gui_kit::theme::set_translucent(ctx, true);
    setup
}

fn glass_off(setup: Setup, ctx: &Context) -> Setup {
    toolbox_gui_kit::theme::set_translucent(ctx, false);
    setup
}

/// The app on its first page, in `theme`, with the desktop blur on or off.
fn home(theme: Theme, blur: bool) -> egui_kittest::Harness<'static, toolbox_gui::App> {
    let setup = Setup {
        look: look_for(shared_runtime(), theme),
        preferences: PreferencesSetup {
            desktop_blur: blur,
            ..PreferencesSetup::default()
        },
        ..Setup::default()
    };

    let fill = if blur { glass_on } else { glass_off };
    launch_with(window(HEIGHT).with_theme(theme), setup, fill)
}

#[test]
fn with_the_glass_the_page_area_paints_no_opaque_window_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = home(theme, true);
        let window = tokens::current(&harness.ctx).window;

        let fills = opaque_fills_at(&harness, IN_THE_PAGE);
        assert!(
            !fills.contains(&window),
            "{theme:?}: opaque window fills {fills:?}"
        );
    }
}

#[test]
fn without_the_glass_the_page_area_keeps_its_opaque_window_fill() {
    for theme in [Theme::Dark, Theme::Light] {
        let harness = home(theme, false);
        let window = tokens::current(&harness.ctx).window;

        let fills = opaque_fills_at(&harness, IN_THE_PAGE);
        assert!(
            fills.contains(&window),
            "{theme:?}: no opaque window fill, got {fills:?}"
        );
    }
}
