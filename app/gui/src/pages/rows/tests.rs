mod divider_gap;
mod gaps;
mod tag_gap;

use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui_kit::testing::painted_text;

use super::actions::{self, CopyName, Named, RowAction, RowKind, Run, Section};
use crate::command::{self, AppCommand, Command, CommandReceiver, CommandSender};

struct Fake(&'static str);

struct Never;

struct Hidden;

struct FakeRows;

impl Named for Fake {
    fn copied_name(&self) -> &str {
        self.0
    }
}

impl RowAction<Fake> for Never {
    fn label(&self, _item: &Fake) -> &'static str {
        "services-start"
    }

    fn possible(&self, _item: &Fake) -> Result<(), &'static str> {
        Err("services-why-masked")
    }

    fn run(&self, _item: &Fake, _run: Run<'_>) {
        unreachable!("a resting action does not run");
    }
}

impl RowAction<Fake> for Hidden {
    fn label(&self, _item: &Fake) -> &'static str {
        "services-stop"
    }

    fn offered(&self, _item: &Fake) -> bool {
        false
    }

    fn run(&self, _item: &Fake, _run: Run<'_>) {}
}

impl RowKind for FakeRows {
    type Item = Fake;

    const SECTIONS: &'static [Section<Fake>] = &[
        Section {
            heading: Some("services-now"),
            actions: &[&Never, &CopyName],
        },
        Section {
            heading: Some("services-with-apply"),
            actions: &[&Hidden],
        },
    ];
}

fn text_rect<State>(harness: &Harness<'_, State>, text: &str) -> Rect {
    let shape = painted_text(harness, text).unwrap_or_else(|| panic!("{text} is painted"));
    shape.galley.rect.translate(shape.pos.to_vec2())
}

fn menu() -> Harness<'static, (CommandSender, CommandReceiver)> {
    let mut harness = toolbox_gui_kit::testing::harness(300.0, 300.0).build_ui_state(
        |ui, (commands, _): &mut (CommandSender, CommandReceiver)| {
            actions::menu::<FakeRows>(ui, &Fake("fake.service"), commands);
        },
        command::channel(),
    );

    harness.run();
    harness
}

#[test]
fn an_action_that_cannot_run_rests_and_says_why() {
    let mut harness = menu();
    let start = harness.get_by_role_and_label(Role::MenuItem, "Start");
    assert!(start.accesskit_node().is_disabled());

    start.hover();
    harness.run_steps(30);

    assert!(
        harness
            .query_by_label("A masked unit cannot start. Unmask it first.")
            .is_some()
    );
}

#[test]
fn a_section_without_an_offered_action_has_no_heading() {
    let harness = menu();

    assert!(harness.query_by_label("Now").is_some());
    assert!(harness.query_by_label("Stop").is_none());
    assert!(harness.query_by_label("With Apply").is_none());
}

#[test]
fn copy_name_copies_and_says_so() {
    let mut harness = menu();

    harness
        .get_by_role_and_label(Role::MenuItem, "Copy name")
        .click();
    harness.step();
    let copied = harness.output().platform_output.commands.iter().any(
        |command| matches!(command, egui::OutputCommand::CopyText(text) if text == "fake.service"),
    );

    harness.run();
    assert!(copied);

    let sent: Vec<Command> = harness.state().1.drain().map(|sent| sent.command).collect();
    assert_eq!(
        sent,
        [Command::from(AppCommand::Toast {
            kind: toolbox_gui_kit::toasts::ToastKind::Info,
            text: "Copied “fake.service” to the clipboard.".to_owned(),
        })]
    );
}

#[test]
fn the_theme_is_asked_for_pictures_of_the_size_the_rows_draw() {
    assert!((f32::from(super::PICTURE_PIXELS) - super::PICTURE_SIZE).abs() < f32::EPSILON);
}
