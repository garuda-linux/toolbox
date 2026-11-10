use std::collections::BTreeSet;

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, HarnessBuilder};
use tokio::runtime::Runtime;
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui::{App, Setup};
use toolbox_plan::PackageName;

pub const WIDTH: f32 = 1024.0;
pub const HEIGHT: f32 = 720.0;
pub const TALL: f32 = 900.0;
pub const SMALLEST: (f32, f32) = (640.0, 480.0);

/// The app and the runtime of its services, which lives as long as the
/// harness.
pub struct Fixture {
    harness: Harness<'static, App>,
    _runtime: Runtime,
}

impl Fixture {
    pub const fn new(harness: Harness<'static, App>, runtime: Runtime) -> Self {
        Self {
            harness,
            _runtime: runtime,
        }
    }

    pub const fn harness(&mut self) -> &mut Harness<'static, App> {
        &mut self.harness
    }
}

pub fn window(height: f32) -> HarnessBuilder<App> {
    toolbox_gui_kit::testing::harness(WIDTH, height)
}

/// The app with `setup` in a [`HEIGHT`] tall window, after its first
/// frames.
pub fn launch(setup: Setup) -> Harness<'static, App> {
    launch_in(window(HEIGHT), setup)
}

pub fn launch_in(builder: HarnessBuilder<App>, setup: Setup) -> Harness<'static, App> {
    let mut harness = builder.build_eframe(move |cc| App::new(cc, setup));
    harness.run();
    harness
}

pub fn packages(names: &[&str]) -> BTreeSet<PackageName> {
    names
        .iter()
        .map(|name| PackageName::new(*name).unwrap())
        .collect()
}

pub fn installing(names: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for name in names {
        changes.want::<Packages>(name.parse().unwrap(), Toggle::On, &Snapshot::default());
    }

    changes
}

pub fn pane(harness: &Harness<'static, App>, name: &str) -> Option<Rect> {
    harness
        .query_by_role_and_label(Role::Pane, name)
        .map(|node| node.rect())
}

pub fn open_console(harness: &mut Harness<'static, App>) -> Rect {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    harness.run();
    pane(harness, "Console").expect("open")
}

pub fn palette(harness: &mut Harness<'static, App>, text: &str) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::K);
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, "Command palette")
        .type_text(text);
    harness.run();
}

pub fn type_into(harness: &mut Harness<'static, App>, field: &str, text: &str) {
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::TextInput, field)
        .type_text(text);
    harness.run();
}

pub fn drag(harness: &mut Harness<'static, App>, from: Pos2, to: Pos2) {
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };

    harness.event(Event::PointerMoved(from));
    harness.step();
    harness.event(button(from, true));
    harness.step();
    for share in 1..=10_u8 {
        harness.event(Event::PointerMoved(from.lerp(to, f32::from(share) / 10.0)));
        harness.step();
    }

    harness.event(button(to, false));
    harness.step();
}

pub fn open_tab(harness: &mut Harness<'static, App>, name: &str) {
    harness
        .get_by_role_and_label(Role::RadioButton, name)
        .click();
    harness.run();
}

pub fn no_note(harness: &Harness<'static, App>) -> bool {
    harness
        .query_by_role_and_label(Role::Alert, "Notification")
        .is_none()
}

pub fn wheel<State>(harness: &Harness<'static, State>, at: Pos2, delta: Vec2) {
    harness.event(Event::PointerMoved(at));
    harness.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta,
        modifiers: Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
}

pub fn checkbox<'harness, State>(
    harness: &'harness Harness<'static, State>,
    title: &'harness str,
) -> egui_kittest::Node<'harness> {
    harness.get_by_role_and_label(Role::CheckBox, title)
}
