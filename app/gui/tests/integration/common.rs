use std::collections::BTreeSet;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, HarnessBuilder};
use tokio::runtime::Runtime;
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui::programs::Programs;
use toolbox_gui::{App, Setup, reader};
use toolbox_plan::PackageName;

use crate::captured::sample_reader;
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
    launch_with(builder, setup, |setup, _| setup)
}
pub fn launch_with(
    builder: HarnessBuilder<App>,
    setup: Setup,
    fill: fn(Setup, &egui::Context) -> Setup,
) -> Harness<'static, App> {
    let sampled = setup.snapshot.is_none();
    let mut harness = builder.build_eframe(move |cc| {
        let setup = finish_reads(fill(setup, &cc.egui_ctx), &cc.egui_ctx);
        App::new(cc, setup)
    });

    harness.run();
    if sampled {
        await_sample_read(&mut harness);
    }

    harness
}

/// `run` stops at the first frame that only asks for a later one, which
/// the loading dots do while the sample read still runs on a busy machine.
fn await_sample_read(harness: &mut Harness<'static, App>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while harness.state().awaits_first_read() {
        assert!(Instant::now() < deadline, "the sample read took over 10 s");
        std::thread::sleep(Duration::from_millis(20));
        harness.step();
    }

    harness.run();
}

/// A reader the test does not give finishes at once, as on a system with
/// nothing to read. Without one a page would wait for its read forever,
/// and its Read again would spin.
fn finish_reads(mut setup: Setup, ctx: &egui::Context) -> Setup {
    if setup.snapshot.is_none() {
        setup.snapshot = Some(sample_reader(ctx, &setup.catalogs, None));
    }

    if setup.programs.is_none() {
        setup.programs = Some(reader::spawn(
            shared_runtime().handle(),
            toolbox_gui::waking::repaint(ctx.clone()),
            "programs",
            Programs::default,
        ));
    }

    setup
}

pub fn shared_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("a runtime for the readers"))
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

pub fn pointer_button(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

pub fn drag(harness: &mut Harness<'static, App>, from: Pos2, to: Pos2) {
    harness.event(Event::PointerMoved(from));
    harness.step();
    harness.event(pointer_button(from, true));
    harness.step();
    for share in 1..=10_u8 {
        harness.event(Event::PointerMoved(from.lerp(to, f32::from(share) / 10.0)));
        harness.step();
    }

    harness.event(pointer_button(to, false));
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

/// A row's button, named by what it does and the row.
pub fn button<'harness, State>(
    harness: &'harness Harness<'static, State>,
    name: &'harness str,
) -> egui_kittest::Node<'harness> {
    harness.get_by_role_and_label(Role::Button, name)
}

pub fn press_ctrl(harness: &mut Harness<'_, App>, key: Key) {
    harness.key_press_modifiers(Modifiers::COMMAND, key);
    harness.run();
}

pub fn focus_terminal(harness: &mut Harness<'_, App>) {
    press_ctrl(harness, Key::J);
    harness
        .get_by_role_and_label(Role::Terminal, "Console output")
        .click();
    harness.run();
}
