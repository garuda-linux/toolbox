//! The pictures of the user guide, one per file in `docs/src/images`. Each
//! draws the fixture of its page, dialog or control in the theme asked for,
//! then keeps the part of the frame the guide talks about.

mod aur;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Rect, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use image::RgbaImage;
use toolbox_gui::page::PageId;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup};

use crate::common::{HEIGHT, TALL, installing, launch_in, open_tab, pane, shared_runtime, window};
use crate::look::look_for;

pub const ALL: [Capture; 26] = [
    Capture {
        name: "about",
        render: about,
    },
    Capture {
        name: "boot",
        render: boot,
    },
    Capture {
        name: "boot-repair",
        render: boot_repair,
    },
    Capture {
        name: "changes-popover",
        render: changes_popover,
    },
    Capture {
        name: "components",
        render: components,
    },
    Capture {
        name: "configs",
        render: configs,
    },
    Capture {
        name: "console-applied",
        render: console_applied,
    },
    Capture {
        name: "diagnostics-report",
        render: diagnostics_report,
    },
    Capture {
        name: "diagnostics-upload",
        render: diagnostics_upload,
    },
    Capture {
        name: "discover",
        render: discover,
    },
    Capture {
        name: "gaming",
        render: gaming,
    },
    Capture {
        name: "guide",
        render: guide,
    },
    Capture {
        name: "home",
        render: home,
    },
    Capture {
        name: "kernels",
        render: kernels,
    },
    Capture {
        name: "locales",
        render: locales,
    },
    Capture {
        name: "maintenance-common",
        render: maintenance_common,
    },
    Capture {
        name: "maintenance-reset",
        render: maintenance_reset,
    },
    Capture {
        name: "network",
        render: network,
    },
    Capture {
        name: "packages-aur-build",
        render: aur::build,
    },
    Capture {
        name: "packages-aur-review",
        render: aur::review,
    },
    Capture {
        name: "palette-open",
        render: palette_open,
    },
    Capture {
        name: "services",
        render: services,
    },
    Capture {
        name: "settings",
        render: settings,
    },
    Capture {
        name: "setup-step",
        render: setup_step,
    },
    Capture {
        name: "system-settings",
        render: system_settings,
    },
    Capture {
        name: "users",
        render: users,
    },
];

/// Room kept around a control, so its shadow and outline stay in the picture.
const MARGIN: f32 = 24.0;

/// A picture of the guide: its file name without the theme's suffix, and
/// its render in a theme, cropped.
pub struct Capture {
    pub name: &'static str,
    pub render: fn(Theme) -> RgbaImage,
}

/// The app's setup with the desktop look of `theme`, for a fixture that has
/// no runtime of its own.
fn launch_setup(theme: Theme) -> Setup {
    Setup {
        look: look_for(shared_runtime(), theme),
        ..Setup::default()
    }
}

/// The frame's part around `rect` and its margin, in the window's own pixels.
fn crop(harness: &mut Harness<'static, App>, rect: Rect, margin: f32) -> RgbaImage {
    let frame = harness.render().expect("the frame renders");
    let scale = harness.ctx.pixels_per_point();
    let rect = rect.expand(margin).intersect(harness.ctx.content_rect());

    let left = pixels(rect.min.x * scale).min(frame.width());
    let top = pixels(rect.min.y * scale).min(frame.height());
    let width = pixels(rect.width() * scale).min(frame.width() - left);
    let height = pixels(rect.height() * scale).min(frame.height() - top);
    image::imageops::crop_imm(&frame, left, top, width, height).to_image()
}

/// A position or size in pixels, rounded and never below zero.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a window is far below u32::MAX pixels, so the rounded value fits"
)]
fn pixels(value: f32) -> u32 {
    value.round().max(0.0) as u32
}

/// The page pane, the whole page under the sidebar and the header.
fn page(harness: &mut Harness<'static, App>) -> RgbaImage {
    let rect = harness.get_by_role_and_label(Role::Pane, "Page").rect();
    crop(harness, rect, 0.0)
}

/// The dialog titled `title`, as large as its own box. The dialog's own
/// accessible node has no bounds, so the box is the top-most area that holds
/// a node of the title.
fn dialog(harness: &mut Harness<'static, App>, title: &str) -> RgbaImage {
    // A dialog rises over a few frames before its box is where it rests.
    for _ in 0..10_u8 {
        harness.step();
    }

    harness.run();
    let point = harness
        .query_all_by_label(title)
        .find(|node| node.accesskit_node().bounding_box().is_some())
        .map(|node| node.rect().center())
        .expect("the dialog's title is shown");

    let ctx = harness.ctx.clone();
    let rect = ctx
        .memory(|memory| {
            memory
                .layer_ids()
                .filter_map(|layer| {
                    memory
                        .area_rect(layer.id)
                        .filter(|area| area.contains(point))
                })
                .last()
        })
        .expect("the dialog's area is laid out");

    crop(harness, rect, MARGIN)
}

/// A named pane, a popover or the Console, as large as its own box.
fn named_pane(harness: &mut Harness<'static, App>, name: &str) -> RgbaImage {
    let rect = pane(harness, name).expect("the pane is shown");
    crop(harness, rect, MARGIN)
}

fn about(theme: Theme) -> RgbaImage {
    let mut harness = launch_in(window(HEIGHT).with_theme(theme), launch_setup(theme));
    harness.get_by_label("About").click();
    harness.run();
    page(&mut harness)
}

fn boot(theme: Theme) -> RgbaImage {
    let mut fixture = crate::boot::app_on(theme);
    let harness = fixture.harness();
    crate::boot::open_boot(harness);
    page(harness)
}

fn boot_repair(theme: Theme) -> RgbaImage {
    let mut fixture = crate::bootrepair::app_on(true, theme);
    let harness = fixture.harness();
    crate::bootrepair::open(harness);
    page(harness)
}

fn changes_popover(theme: Theme) -> RgbaImage {
    let setup = Setup {
        queue: Queue::in_memory(installing(&["firefox", "vlc"])),
        ..launch_setup(theme)
    };

    let mut harness = launch_in(window(HEIGHT).with_theme(theme), setup);
    harness.get_by_label("Review").click();
    for _ in 0..6_u8 {
        harness.step();
    }

    harness.run();
    named_pane(&mut harness, "Changes to apply")
}

fn components(theme: Theme) -> RgbaImage {
    let system = crate::snapshots::read_in_full(crate::components::with_pipewire());
    let (mut harness, _runtime) = crate::components::components_on(system, theme);
    page(&mut harness)
}

fn configs(theme: Theme) -> RgbaImage {
    let mut fixture = crate::configs::configs_on(theme);
    page(fixture.harness())
}

fn console_applied(theme: Theme) -> RgbaImage {
    let script = crate::console::succeeding_script();
    let (mut harness, _runtime) = crate::console_output::applied_with(script, false, theme);

    // Without the focus the terminal's cursor is hidden, as in the snapshots.
    crate::console::unfocus(&mut harness);
    named_pane(&mut harness, "Console")
}

fn diagnostics_report(theme: Theme) -> RgbaImage {
    let mut fixture = crate::diagnostics::diagnostics_page_on(false, theme);
    let harness = fixture.harness();
    crate::diagnostics::collect(harness);
    page(harness)
}

fn diagnostics_upload(theme: Theme) -> RgbaImage {
    let mut fixture = crate::diagnostics::diagnostics_page_on(false, theme);
    let harness = fixture.harness();
    crate::diagnostics::collect(harness);
    harness.get_by_label("Share Report…").click();
    harness.run();
    dialog(harness, "Upload This Report?")
}

fn discover(theme: Theme) -> RgbaImage {
    let (mut harness, _runtime) = crate::discover::pictured_on(
        crate::discover::source_catalogs(),
        crate::discover::system(),
        None,
        theme,
    );

    page(&mut harness)
}

fn gaming(theme: Theme) -> RgbaImage {
    let (mut harness, _runtime) = crate::gaming::gaming_on(theme);
    page(&mut harness)
}

fn guide(theme: Theme) -> RgbaImage {
    let setup = Setup {
        open: Some(PageId::Kernels),
        ..launch_setup(theme)
    };

    let mut harness = launch_in(window(HEIGHT).with_theme(theme), setup);
    harness.key_press(Key::F1);
    harness.run();
    page(&mut harness)
}

fn home(theme: Theme) -> RgbaImage {
    let (mut harness, _runtime, _power) = crate::home::home_with_on(
        crate::home::host(false, true),
        crate::snapshots::behind_home(),
        crate::home::quiet_facts(),
        theme,
    );

    page(&mut harness)
}

fn kernels(theme: Theme) -> RgbaImage {
    let (mut harness, _runtime) = crate::kernels::kernels_on(crate::kernels::system(true), theme);
    page(&mut harness)
}

fn locales(theme: Theme) -> RgbaImage {
    let mut fixture = crate::locales::languages_on(theme);
    page(fixture.harness())
}

fn maintenance_common(theme: Theme) -> RgbaImage {
    let mut fixture = crate::maintenance::maintenance_on(theme);
    page(fixture.harness())
}

fn maintenance_reset(theme: Theme) -> RgbaImage {
    let mut fixture = crate::maintenance::maintenance_on(theme);
    let harness = fixture.harness();
    open_tab(harness, "Reset Settings");
    harness
        .get_by_role_and_label(Role::CheckBox, "Fish")
        .click();
    harness
        .get_by_role_and_label(Role::CheckBox, "Kitty")
        .click();
    harness.get_by_label("Reset Selected…").click();
    harness.run();
    dialog(harness, "Reset 2 Settings?")
}

fn network(theme: Theme) -> RgbaImage {
    let mut fixture = crate::network::network_on(theme);
    page(fixture.harness())
}

fn palette_open(theme: Theme) -> RgbaImage {
    let mut harness = launch_in(window(TALL).with_theme(theme), launch_setup(theme));
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    dialog(&mut harness, "Command Palette")
}

fn services(theme: Theme) -> RgbaImage {
    let (mut harness, _runtime) = crate::services::services_on(theme);
    page(&mut harness)
}

fn settings(theme: Theme) -> RgbaImage {
    let (mut harness, _store) = crate::settings::settings_on(theme);
    page(&mut harness)
}

fn setup_step(theme: Theme) -> RgbaImage {
    let mut fixture = crate::setup::assistant_on(theme);
    let harness = fixture.harness();
    crate::setup::pick_strawberry(harness);
    page(harness)
}

fn system_settings(theme: Theme) -> RgbaImage {
    let system = crate::snapshots::read_in_full(crate::tweaks::system());
    let (mut harness, _runtime) = crate::tweaks::tweaks_on(system, theme);
    page(&mut harness)
}

fn users(theme: Theme) -> RgbaImage {
    let mut fixture = crate::users::users_on(theme);
    page(fixture.harness())
}
