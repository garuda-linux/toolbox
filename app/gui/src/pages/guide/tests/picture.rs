use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::page;
use crate::pages::guide::markdown::Reader;
use crate::pages::guide::picture::show;

const ALT: &str = "The Config Files page with its file list";

/// The picture keeps its room from the first frame, so the words under
/// it never move while it is decoded.
#[test]
fn a_picture_keeps_its_room_before_it_is_shown() {
    let mut harness = page();
    harness.state_mut().page.show("config-files.md", None);
    for _ in 0_u8..5 {
        if harness.query_by_role_and_label(Role::Label, ALT).is_some() {
            break;
        }

        harness.step();
    }

    let first = harness.get_by_role_and_label(Role::Label, ALT).rect().top();
    harness.run();
    let settled = harness.get_by_role_and_label(Role::Label, ALT).rect().top();

    assert!(
        (first - settled).abs() < 0.5,
        "the words under the picture moved from {first} to {settled}"
    );
}

#[test]
fn a_click_shows_the_picture_large_and_escape_leads_back() {
    let mut harness = page();
    harness.state_mut().page.show("config-files.md", None);
    harness.run();

    // The picture starts below the window's edge.
    harness
        .get_by_role_and_label(Role::Button, &format!("Show larger: {ALT}"))
        .scroll_to_me();
    harness.run();

    let small = harness
        .get_by_role_and_label(Role::Button, &format!("Show larger: {ALT}"))
        .rect();

    harness
        .get_by_role_and_label(Role::Button, &format!("Show larger: {ALT}"))
        .click();

    harness.run();

    // The dialog's node has no rect of its own, its picture has.
    let dialog = harness.get_by_role_and_label(Role::Dialog, ALT);
    let large = dialog
        .query_all_by_role(Role::Image)
        .map(|node| node.rect())
        .reduce(|wider, rect| {
            if rect.width() > wider.width() {
                rect
            } else {
                wider
            }
        })
        .expect("the dialog shows the picture");

    // A wide column can make the picture taller than the window. The larger
    // view then takes the window's height less the dialog's frame.
    let window = harness.ctx.content_rect().height();
    assert!(
        large.height() >= window * 0.8,
        "the larger view {large:?} does not use the window {window} tall, the page shows {small:?}"
    );

    harness.key_press(egui::Key::Escape);
    harness.run();

    assert!(harness.query_by_role(Role::Dialog).is_none());
}

#[test]
fn a_picture_has_an_outline_of_its_own() {
    let mut harness = page();
    harness.state_mut().page.show("config-files.md", None);
    harness.run();

    let picture = harness
        .get_by_role_and_label(Role::Button, &format!("Show larger: {ALT}"))
        .rect();

    let outline = toolbox_gui_kit::tokens::current(&harness.ctx).outline;

    let framed = toolbox_gui_kit::testing::rims_in(&harness, outline)
        .into_iter()
        .any(|(rim, width)| (width - 1.0).abs() < 0.01 && rim.expand(2.0).contains_rect(picture));

    assert!(framed, "no 1 px outline round {picture:?}");
}

/// The picture's button says the description, so the caption under it
/// stays out of a screen reader's way.
#[test]
fn a_screen_reader_hears_the_description_once() {
    let mut harness = page();
    harness.state_mut().page.show("config-files.md", None);
    harness.run();

    let caption = harness.get_by_role_and_label(Role::Label, ALT);

    assert!(caption.accesskit_node().is_hidden());
    assert!(
        harness
            .query_by_role_and_label(Role::Button, &format!("Show larger: {ALT}"))
            .is_some()
    );
}

#[test]
fn a_missing_picture_leaves_a_placeholder_over_its_spoken_caption() {
    let image = toolbox_guide::Image {
        src: "images/gone.png".to_owned(),
        alt: "A picture that is gone".to_owned(),
    };

    let harness = toolbox_gui_kit::testing::harness(600.0, 400.0).build_ui(move |ui| {
        let mut reader = Reader::new(None);
        show(ui, &image, &mut reader);
    });

    let placeholder = harness.get_by_label("The picture is missing.").rect();
    let caption = harness.get_by_label("A picture that is gone");

    assert!(!caption.accesskit_node().is_hidden());
    assert!(caption.rect().top() > placeholder.bottom());
    assert!(harness.query_by_role(Role::Button).is_none());
}
