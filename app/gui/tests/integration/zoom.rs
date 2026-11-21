use std::collections::BTreeMap;

use egui::{Key, ViewportCommand, ViewportId};
use egui_kittest::Harness;
use toolbox_gui::command::{self, CommandBus, UiCommand};
use toolbox_gui::{App, Setup};

use crate::common::{HEIGHT, focus_terminal, launch, press_ctrl as press, window};

#[derive(Default)]
struct Stored(BTreeMap<String, String>);

impl eframe::Storage for Stored {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

fn zoom(harness: &Harness<'_, App>) -> f32 {
    harness.ctx.zoom_factor()
}

fn near(left: f32, right: f32) -> bool {
    (left - right).abs() < 1e-4
}

#[test]
fn ctrl_plus_and_minus_zoom_in_tenths_and_ctrl_0_resets() {
    let mut harness = launch(Setup::default());

    press(&mut harness, Key::Plus);
    assert!(near(zoom(&harness), 1.1), "{}", zoom(&harness));

    press(&mut harness, Key::Equals);
    assert!(near(zoom(&harness), 1.2), "Ctrl and = without Shift");

    press(&mut harness, Key::Minus);
    assert!(near(zoom(&harness), 1.1));

    press(&mut harness, Key::Num0);
    assert!(near(zoom(&harness), 1.0));
}

#[test]
fn the_zoom_stops_at_its_bounds() {
    let mut harness = launch(Setup::default());

    for _ in 0_u8..12 {
        press(&mut harness, Key::Plus);
    }

    assert!(near(zoom(&harness), 1.5), "{}", zoom(&harness));

    for _ in 0_u8..12 {
        press(&mut harness, Key::Minus);
    }

    assert!(near(zoom(&harness), 0.8), "{}", zoom(&harness));
}

/// The pages are laid out for the smallest window in points, so it grows
/// in pixels with the zoom.
#[test]
fn a_zoom_keeps_the_smallest_window_in_points() {
    let (sender, receiver) = command::channel();
    let mut harness = launch(Setup {
        commands: Some(CommandBus {
            sender: sender.clone(),
            receiver,
        }),
        ..Setup::default()
    });

    // A key press would take one frame per event, and the harness keeps
    // the output of the last frame only.
    sender.send(UiCommand::ZoomIn);
    harness.step();
    harness.step();

    let commands = &harness.output().viewport_output[&ViewportId::ROOT].commands;
    let kept = ViewportCommand::MinInnerSize(toolbox_gui::zoom::SMALLEST_WINDOW);
    assert!(commands.contains(&kept), "{commands:?}");
}

#[test]
fn the_zoom_survives_a_save_and_a_new_start() {
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::Plus);
    press(&mut harness, Key::Plus);

    let mut stored = Stored::default();
    eframe::App::save(harness.state_mut(), &mut stored);
    let stored: &'static Stored = Box::leak(Box::new(stored));

    let mut restarted = window(HEIGHT).build_eframe(move |cc| {
        cc.storage = Some(stored);
        App::new(cc, Setup::default())
    });

    restarted.run();

    assert!(near(zoom(&restarted), 1.2), "{}", zoom(&restarted));
}

#[test]
fn a_focused_terminal_keeps_the_zoom_keys() {
    let mut harness = launch(Setup::default());
    press(&mut harness, Key::Plus);
    focus_terminal(&mut harness);

    for key in [Key::Plus, Key::Equals, Key::Minus, Key::Num0] {
        press(&mut harness, key);
        assert!(near(zoom(&harness), 1.1), "{key:?}");
    }
}
