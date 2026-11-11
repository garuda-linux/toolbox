use egui::accesskit::Role;
use egui::{Key, Modifiers, Rect};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup};

use crate::common::{HEIGHT, installing, launch, launch_in, palette, pane, window};

fn queued(changes: ChangeSet) -> Setup {
    Setup {
        queue: Queue::in_memory(changes),
        ..Setup::default()
    }
}

pub fn app_with(packages: &[&str]) -> Harness<'static, App> {
    launch(queued(installing(packages)))
}

/// The app at sixty frames a second, so a rise is seen frame by frame.
fn moving_app(packages: &[&str]) -> Harness<'static, App> {
    let sixty = window(HEIGHT).with_step_dt(1.0 / 60.0).with_max_steps(120);
    launch_in(sixty, queued(installing(packages)))
}

fn review(harness: &mut Harness<'static, App>) {
    harness.get_by_label("Review").click();
    harness.run();
}

#[test]
fn an_empty_queue_shows_no_pill() {
    let harness = app_with(&[]);

    assert!(harness.query_by_label_contains("queued").is_none());
}

#[test]
fn the_pill_counts_and_its_popover_lists_every_change_in_words() {
    let mut harness = app_with(&["firefox", "vlc"]);
    assert!(harness.query_by_label("changes queued").is_some());
    assert!(
        harness.query_by_label("Install firefox").is_none(),
        "closed"
    );

    review(&mut harness);

    assert!(harness.query_by_label("Queued changes").is_some());
    assert!(harness.query_by_label("Install firefox").is_some());
    assert!(harness.query_by_label("Install vlc").is_some());
    assert!(harness.query_by_label("Apply 2 changes").is_some());
}

#[test]
fn one_change_is_removed_by_its_own_button() {
    let mut harness = app_with(&["firefox", "vlc"]);
    review(&mut harness);

    harness.get_by_label("Remove “Install firefox”").click();
    harness.run();

    assert!(harness.query_by_label("Install firefox").is_none());
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn removing_all_asks_first_and_keeping_them_changes_nothing() {
    let mut harness = app_with(&["firefox", "vlc"]);
    review(&mut harness);

    harness.get_by_label("Remove all…").click();
    harness.run();
    assert!(
        harness
            .query_by_label("Remove all 2 queued changes?")
            .is_some()
    );

    harness.get_by_label("Keep them").click();
    harness.run();
    assert!(harness.query_by_label("Install vlc").is_some());
}

#[test]
fn removing_all_after_the_question_empties_the_queue() {
    let mut harness = app_with(&["firefox", "vlc"]);
    review(&mut harness);

    harness.get_by_label("Remove all…").click();
    harness.run();
    harness.get_by_label("Remove all").click();
    harness.run();

    assert!(harness.query_by_label_contains("queued").is_none());
}

/// Where the open popover lies on its third frame, and once it rests. Its
/// area measures itself on the first and places itself by that guess on
/// the second.
fn popover_rise(harness: &mut Harness<'static, App>) -> (Rect, Rect) {
    let mut seen = Vec::new();
    for _ in 0..6_u8 {
        harness.step();
        seen.extend(popover(harness));
    }

    harness.run();
    let drawn = *seen.get(2).expect("shows");
    (drawn, popover(harness).expect("rests"))
}

fn popover(harness: &Harness<'static, App>) -> Option<Rect> {
    pane(harness, "Changes to apply")
}

#[test]
fn a_click_on_review_lets_the_popover_rise() {
    let mut harness = moving_app(&["firefox"]);

    harness.get_by_label("Review").click();
    let (first, rest) = popover_rise(&mut harness);

    assert!(first.bottom() > rest.bottom() + 0.5, "{first:?} {rest:?}");
}

#[test]
fn enter_on_review_shows_the_popover_at_once() {
    let mut harness = moving_app(&["firefox"]);

    harness.get_by_label("Review").focus();
    harness.run();
    harness.key_press(Key::Enter);
    let (first, rest) = popover_rise(&mut harness);

    assert_eq!(first, rest);
}

#[test]
fn space_on_review_shows_the_popover_at_once() {
    let mut harness = moving_app(&["firefox"]);

    harness.get_by_label("Review").focus();
    harness.run();
    harness.key_press(Key::Space);
    let (first, rest) = popover_rise(&mut harness);

    assert_eq!(first, rest);
}

fn review_rows(harness: &Harness<'static, App>) -> usize {
    harness
        .query_all_by_label_contains("Review changes")
        .count()
}

#[test]
fn the_palette_offers_review_only_with_queued_changes() {
    let mut empty = app_with(&[]);
    palette(&mut empty, "review");
    assert_eq!(review_rows(&empty), 0);

    let mut queued = app_with(&["firefox"]);
    let before = review_rows(&queued);
    palette(&mut queued, "review");
    assert_eq!(review_rows(&queued), before + 1);
}

#[test]
fn a_palette_row_without_a_shortcut_is_named_by_its_title_alone() {
    let mut harness = app_with(&["firefox"]);
    palette(&mut harness, "review");

    let names: Vec<String> = harness
        .query_all_by_label_contains("Review changes")
        .filter_map(|node| node.accesskit_node().label())
        .collect();

    assert!(!names.is_empty(), "no row");
    assert!(names.iter().all(|name| name == name.trim()), "{names:?}");
}

#[test]
fn enter_in_the_palette_shows_the_popover_at_once() {
    let mut harness = moving_app(&["firefox"]);
    palette(&mut harness, "review");

    harness.key_press(Key::Enter);
    let (first, rest) = popover_rise(&mut harness);

    assert_eq!(first, rest);
}

#[test]
fn a_click_on_the_palette_row_lets_the_popover_rise() {
    let mut harness = moving_app(&["firefox"]);
    palette(&mut harness, "review");

    // The palette lies over the page, so its row is the last one drawn.
    harness
        .query_all_by_label_contains("Review changes")
        .last()
        .expect("the palette's row")
        .click();
    let (first, rest) = popover_rise(&mut harness);

    assert!(first.bottom() > rest.bottom() + 0.5, "{first:?} {rest:?}");
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Command palette")
            .is_none(),
        "the palette's row ran it"
    );
}

#[test]
fn the_one_password_note_is_secondary_text() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);
    let secondary = toolbox_gui_kit::tokens::current(&harness.ctx).secondary;

    assert_eq!(
        toolbox_gui_kit::testing::text_colour(&harness, "Applied together, one password"),
        Some(secondary)
    );
}

/// The pill's own band below the page, so it never lies over a control.
#[test]
fn the_pill_lies_below_the_page_never_over_it() {
    let harness = app_with(&["firefox"]);
    let page = harness.get_by_role_and_label(Role::Pane, "Page").rect();
    let pill = harness.get_by_label("Review").rect();

    assert!(
        pill.top() >= page.bottom(),
        "the pill at {pill:?} lies over the page at {page:?}"
    );
}

#[test]
fn an_empty_queue_leaves_the_page_the_whole_height() {
    let with = app_with(&["firefox"]);
    let without = app_with(&[]);
    let page =
        |harness: &Harness<'static, App>| harness.get_by_role_and_label(Role::Pane, "Page").rect();

    assert!(page(&without).bottom() > page(&with).bottom());
}

/// With the Console open, the queue is a compact group at the right of its
/// header, beside Close, without the pill's words.
#[test]
fn the_open_console_holds_the_queue_beside_its_close_button() {
    let mut harness = app_with(&["firefox", "vlc"]);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::J);
    harness.run();

    let close = harness.get_by_label("Hide the Console").rect();
    let apply = harness.get_by_label("Apply").rect();
    let review = harness.get_by_label("Review").rect();

    assert!(harness.query_by_label("changes queued").is_none());
    assert!(review.right() <= apply.left() && apply.right() <= close.left());
    assert!(
        close.left() - apply.right() < 40.0,
        "Apply at {apply:?}, Close at {close:?}"
    );
    assert!((apply.center().y - close.center().y).abs() < 4.0);
}

#[test]
fn the_pill_keeps_its_width_when_review_turns_into_hide() {
    let mut harness = app_with(&["firefox"]);
    let closed = harness.get_by_role_and_label(Role::Button, "Review").rect();
    review(&mut harness);
    let open = harness.get_by_role_and_label(Role::Button, "Hide").rect();

    assert!(
        (closed.width() - open.width()).abs() < 0.5,
        "Review is {} wide, Hide {}",
        closed.width(),
        open.width()
    );
}

#[test]
fn the_pill_and_its_popover_are_frosted_not_solid() {
    let mut harness = app_with(&["firefox"]);
    let pill = harness.get_by_label("Review").rect();
    let frosted = |shown: &Harness<'static, App>, near: Rect| {
        let textured = toolbox_gui_kit::testing::textured_rects(shown);
        toolbox_gui_kit::testing::filled_rects(shown, toolbox_gui_kit::frost::raised(&shown.ctx))
            .iter()
            .any(|rect| rect.contains_rect(near) && textured.contains(rect))
    };

    assert!(frosted(&harness, pill), "the pill is solid");

    review(&mut harness);
    let open = popover(&harness).expect("the popover opens");
    assert!(frosted(&harness, open.shrink(4.0)), "the popover is solid");
}

#[test]
fn apply_with_a_removal_queued_first_shows_what_goes() {
    let mut removing = ChangeSet::default();
    removing.want::<Packages>("steam".parse().unwrap(), Toggle::Off, &Snapshot::default());
    let mut harness = launch(queued(removing));

    harness.get_by_role_and_label(Role::Button, "Apply").click();
    harness.run();

    assert!(
        harness.query_by_label("Queued changes").is_some(),
        "the list of changes opens before anything is removed"
    );
}

#[test]
fn escape_closes_the_list_and_review_is_focused_again() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);
    assert!(harness.query_by_label("Queued changes").is_some());

    harness.key_press(Key::Escape);
    harness.run();

    assert!(harness.query_by_label("Queued changes").is_none());
    assert!(harness.get_by_label("Review").is_focused());
}

#[test]
fn with_the_list_open_one_apply_button_is_left() {
    let mut harness = app_with(&["firefox"]);
    review(&mut harness);

    let applies = harness
        .get_all_by_role(Role::Button)
        .filter(|node| {
            node.accesskit_node()
                .label()
                .is_some_and(|label| label.starts_with("Apply"))
        })
        .count();

    assert_eq!(applies, 1);
}

#[test]
fn a_change_whose_last_apply_failed_is_tagged_in_the_popover() {
    let mut changes = installing(&["firefox", "vlc"]);
    let (_, owners) = toolbox_core::planned(&changes);
    changes.finished(&owners, |_| false);
    let mut harness = launch(queued(changes));

    review(&mut harness);

    assert!(harness.query_by_label("Install firefox").is_some());
    assert_eq!(harness.query_all_by_label("Did not finish").count(), 2);
}

#[test]
fn an_untried_change_has_no_tag() {
    let mut harness = app_with(&["firefox"]);

    review(&mut harness);

    assert!(harness.query_by_label("Did not finish").is_none());
}

/// A queue too long for the room above the pill keeps a gap to the
/// window's top and scrolls a shorter list.
#[test]
fn a_long_queue_in_a_short_window_keeps_off_the_top_edge() {
    let names: Vec<String> = (0_u8..30).map(|n| format!("package{n}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let short = toolbox_gui_kit::testing::harness(crate::common::WIDTH, 400.0);
    let mut harness = launch_in(short, queued(installing(&names)));
    review(&mut harness);

    let sizes = toolbox_gui_kit::tokens::current(&harness.ctx).sizes;
    let title = harness.get_by_label("Queued changes").rect();
    let least = sizes.popover_gap + f32::from(sizes.popover_padding.top);

    assert!(
        title.top() >= least - 0.5,
        "the title is at {}",
        title.top()
    );
}
