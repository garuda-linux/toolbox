//! Where the guide's links and its scroll lead: the end of a page marks
//! its last heading, a missing page or heading leaves no trace behind.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::{Page, page_on, sent};
use crate::command::{AppCommand, Command};

fn marked(harness: &Page, label: &str) -> bool {
    harness
        .query_all_by_role_and_label(Role::Link, label)
        .any(|link| link.accesskit_node().is_selected() == Some(true))
}

fn wheel_to_the_end(harness: &mut Page) {
    let article = harness
        .query_all_by_role(Role::Heading)
        .next()
        .expect("the page has a title")
        .rect();

    harness.event(egui::Event::PointerMoved(article.center()));
    for _ in 0_u8..40 {
        harness.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -400.0),
            modifiers: egui::Modifiers::NONE,
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
        });
        harness.step();
    }

    harness.run();
}

/// "See also" is two lines long, so its heading never reaches the top of
/// the view.
#[test]
fn the_end_of_a_page_marks_its_last_heading() {
    let mut harness = page_on("settings.md");
    harness.set_size(egui::vec2(1440.0, 900.0));
    harness.run();
    assert!(marked(&harness, "Settings"), "the first heading at the top");

    wheel_to_the_end(&mut harness);

    assert!(marked(&harness, "See also"));
    assert!(!marked(&harness, "About"));
}

#[test]
fn a_missing_page_opens_the_first_and_says_so() {
    let mut harness = page_on("gone.md");

    assert_eq!(
        harness.state().page.file,
        toolbox_guide::FIRST_PAGE,
        "the first page opened"
    );
    assert!(sent(&harness).iter().any(|command| matches!(
        command,
        Command::App(AppCommand::Toast { kind: _, text }) if text.contains("gone.md")
    )));

    harness.run();
    let toasts = sent(&harness)
        .iter()
        .filter(|command| {
            matches!(
                command,
                Command::App(AppCommand::Toast { kind: _, text: _ })
            )
        })
        .count();

    assert_eq!(toasts, 0, "the toast came again");
}

#[test]
fn a_missing_heading_is_forgotten_after_one_frame() {
    let mut harness = page_on("kernels.md");

    let page = &mut harness.state_mut().page;
    page.show("kernels.md", Some("no-such-heading"));
    harness.run();

    assert_eq!(harness.state().page.target, None);
}
