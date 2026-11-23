//! The footer under the page, as KDE's footer toolbar: the whole page
//! width at the page's bottom, the text at the left, the buttons at the
//! right and the padding of `KCMUtils` footer.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::tokens;

use super::{app_with, bar_rect, footer_id};
use crate::common::pane;

/// Darkly's push button at the test font, its text plus 18 px.
const PUSH_HEIGHT: f32 = 34.0;

/// `KCMUtils` footer padding (`margins`, Breeze's `Layout_ChildMarginWidth`),
/// on every side of the buttons.
const KCM_MARGIN: f32 = 6.0;

/// The page body's side margin, which the footer's row lines up with.
const PAGE_SIDE: f32 = 18.0;

/// The margins above and below the buttons together.
const KCM_MARGINS: f32 = KCM_MARGIN + KCM_MARGIN;

/// A button's frame and its shadow reach a pixel or two past its rect.
const EDGE_SLACK: f32 = 2.0;

#[test]
fn the_footer_spans_the_page_at_its_bottom() {
    let harness = app_with(&["firefox"]);
    let page = pane(&harness, "Page").expect("the page");
    let bar = bar_rect(&harness);

    assert!(
        (bar.left() - page.left()).abs() < 0.5 && (bar.right() - page.right()).abs() < 0.5,
        "the footer {bar:?} is not as wide as the page {page:?}"
    );
    assert!(
        (bar.top() - page.bottom()).abs() < 0.5,
        "the footer {bar:?} does not start where the page {page:?} ends"
    );
}

#[test]
fn the_footer_is_as_tall_as_kcm_draws_it() {
    let harness = app_with(&["firefox"]);
    let sizes = tokens::current(&harness.ctx).sizes;
    let bar = bar_rect(&harness);
    let apply = harness.get_by_role_and_label(Role::Button, "Apply").rect();

    assert!(
        (bar.height() - sizes.changes_chrome - apply.height()).abs() < 0.5,
        "the footer is {} tall",
        bar.height()
    );
    assert!(
        (apply.height() - PUSH_HEIGHT).abs() < 0.5,
        "Apply is {} tall",
        apply.height()
    );
    assert!(
        (bar.height() - (1.0 + KCM_MARGINS + PUSH_HEIGHT)).abs() < 0.5,
        "one separator, the margins and the button make {} px",
        bar.height()
    );
}

#[test]
fn the_text_is_at_the_left_and_the_buttons_at_the_right() {
    let harness = app_with(&["firefox", "vlc"]);
    let bar = bar_rect(&harness);
    let text = harness.get_by_label("2 changes queued").rect();
    let review = harness.get_by_role_and_label(Role::Button, "Review").rect();
    let apply = harness.get_by_role_and_label(Role::Button, "Apply").rect();

    assert!(
        (text.left() - bar.left() - PAGE_SIDE).abs() < EDGE_SLACK,
        "the text at {text:?} is not at the left of {bar:?}"
    );
    assert!(text.right() <= review.left(), "{text:?} over {review:?}");
    assert!(review.right() <= apply.left(), "{review:?} over {apply:?}");
    assert!(
        (bar.right() - apply.right() - PAGE_SIDE).abs() < EDGE_SLACK,
        "Apply at {apply:?} is not {PAGE_SIDE} from the right of {bar:?}"
    );
    assert!(
        (bar.bottom() - apply.bottom() - KCM_MARGIN).abs() < EDGE_SLACK,
        "Apply at {apply:?} is not {KCM_MARGIN} from the bottom of {bar:?}"
    );
}

/// With no queue the footer has no area, so the page keeps its whole height
/// and no bar is drawn.
#[test]
fn an_empty_queue_has_no_footer_area() {
    let without = app_with(&[]);

    assert!(
        without
            .ctx
            .memory(|memory| memory.area_rect(footer_id()))
            .is_none()
    );
}
