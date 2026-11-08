mod status_word;

use egui_kittest::kittest::Queryable as _;

use super::super::{Tone, banner};
use crate::testing::{harness, rims_around};

#[test]
fn a_banner_shows_its_actions_in_order_and_tells_which_was_clicked() {
    let mut harness = harness(800.0, 200.0).build_ui_state(
        |ui, clicked: &mut Option<usize>| {
            let actions = ["Show details", "Read again"];
            if let Some(index) = banner(ui, Tone::Warning, "3 checks failed.", &actions) {
                *clicked = Some(index);
            }
        },
        None,
    );

    harness.run();

    let details = harness.get_by_label("Show details").rect();
    let again = harness.get_by_label("Read again").rect();

    assert!(details.right() <= again.left(), "{details:?} {again:?}");

    harness.get_by_label("Read again").click();
    harness.run();

    assert_eq!(*harness.state(), Some(1));
}

#[test]
fn a_banner_without_actions_has_no_button() {
    let mut harness = harness(800.0, 200.0)
        .build_ui(|ui| assert_eq!(banner(ui, Tone::Neutral, "Note.", &[]), None));

    harness.run();

    assert!(
        harness
            .query_by_role(egui::accesskit::Role::Button)
            .is_none()
    );
}

#[test]
fn a_banner_action_keeps_an_outline_on_the_neutral_wash() {
    // The neutral wash and a Soft button at rest share one fill.
    let mut harness = harness(800.0, 200.0)
        .build_ui(|ui| _ = banner(ui, Tone::Neutral, "garuda-update is missing.", &["Install"]));

    harness.run();

    let button = harness.get_by_label("Install").rect();
    let rims = rims_around(&harness, button.center());

    assert!(rims.iter().any(|width| *width >= 1.0), "{rims:?}");
}

#[test]
fn a_success_banner_shows_the_positive_glyph() {
    let tokens = crate::tokens::current(&egui::Context::default());
    let (_, _, icon) = Tone::Success.banner(&tokens);

    assert_eq!(icon, crate::icons::DIALOG_POSITIVE);
}

#[test]
fn a_one_line_banner_is_as_tall_with_or_without_an_action() {
    let mut heights = Vec::new();
    for actions in [&[][..], &["Queue all"][..]] {
        let mut harness = harness(800.0, 200.0).build_ui_state(
            |ui, height: &mut f32| {
                let response = ui.scope(|ui| _ = banner(ui, Tone::Warning, "One line.", actions));
                *height = response.response.rect.height();
            },
            0.0,
        );

        harness.run();
        heights.push(*harness.state());
    }

    assert_eq!(heights[0], heights[1]);
}

#[test]
fn a_banner_is_announced_and_an_error_one_at_once() {
    use egui::accesskit::{Live, Role};
    use egui_kittest::kittest::NodeT as _;

    let harness = harness(800.0, 200.0).build_ui(|ui| {
        _ = banner(ui, Tone::Warning, "3 checks failed.", &[]);
        _ = banner(ui, Tone::Error, "The update failed.", &[]);
    });

    let warning = harness.get_by_label("3 checks failed.");
    assert_eq!(warning.accesskit_node().live(), Live::Polite);

    let error = harness.get_by_label("The update failed.");
    assert_eq!(error.accesskit_node().live(), Live::Assertive);
    assert_eq!(error.accesskit_node().role(), Role::Alert);
}

/// Where the last frame painted the banner's icon, the only picture.
fn icon_rect<State>(harness: &egui_kittest::Harness<'_, State>) -> egui::Rect {
    *crate::testing::textured_rects(harness)
        .first()
        .expect("the banner shows its icon")
}

#[test]
fn a_banner_icon_is_centred_on_its_whole_text() {
    let text = "A mirror no longer has a file this install needs: the package databases are \
                older than the mirror. A system update brings them up to date.";

    for actions in [&[][..], &["Update the system and try again"][..]] {
        let mut harness =
            harness(480.0, 300.0).build_ui(|ui| _ = banner(ui, Tone::Warning, text, actions));

        harness.run();

        let words = harness.get_by_label(text).rect();
        let icon = icon_rect(&harness);

        assert!(words.height() > 30.0, "the text is {words:?}");
        assert!(
            (icon.center().y - words.center().y).abs() < 1.0,
            "the icon {icon:?} beside the text {words:?}"
        );
    }
}

#[test]
fn a_one_line_banner_centres_its_icon_on_the_line() {
    let mut harness =
        harness(800.0, 200.0).build_ui(|ui| _ = banner(ui, Tone::Warning, "One line.", &["Queue"]));

    harness.run();

    let words = harness.get_by_label("One line.").rect();
    let icon = icon_rect(&harness);

    assert!(
        (icon.center().y - words.center().y).abs() < 1.0,
        "the icon {icon:?} beside the text {words:?}"
    );
}

#[test]
fn a_banner_action_on_a_status_wash_is_filled_without_a_rim() {
    let mut harness = harness(800.0, 200.0)
        .build_ui(|ui| _ = banner(ui, Tone::Warning, "3 checks failed.", &["Read again"]));

    harness.run();

    let button = harness.get_by_label("Read again").rect();
    let field = crate::tokens::current(&harness.ctx).field;

    assert_eq!(
        crate::testing::fill_under(&harness, button.center()),
        Some(field)
    );
    let rims = rims_around(&harness, button.center());
    assert!(rims.iter().all(|width| *width < 0.5), "{rims:?}");
}
