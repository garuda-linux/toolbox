use std::path::Path;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::domains::Packages;
use toolbox_core::features::console::ending::Ending;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Toggle};
use toolbox_plan::PackageName;

use super::{SetupPage, Stage};
use crate::catalogs::Catalogs;
use crate::command::{Command, HomeCommand, UiCommand};
use crate::pages::Plans;
use crate::testing::{PageFixture, PageState};

type Shown = Harness<'static, PageState<SetupPage>>;

fn page(changes: ChangeSet, plans: Plans, stage: Stage) -> Shown {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    let start = SetupPage {
        stage,
        review_at: Some(0),
        ..SetupPage::default()
    };

    let mut harness = PageFixture {
        catalogs: Catalogs::load(&dir),
        changes,
        plans,
        ..PageFixture::default()
    }
    .harness(start);

    harness.run();
    harness
}

fn first() -> Shown {
    page(ChangeSet::default(), Plans::default(), Stage::Welcome)
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn shows(harness: &Shown, label: &str) -> bool {
    harness.query_all_by_label_contains(label).next().is_some()
}

fn strawberry() -> ChangeSet {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(
        PackageName::new("strawberry").unwrap(),
        Toggle::On,
        &toolbox_core::Snapshot::default(),
    );
    changes
}

#[test]
fn the_welcome_counts_the_steps_and_next_opens_the_first() {
    let mut harness = first();

    assert!(shows(&harness, "There are 16 categories"));

    harness.get_by_label("Next").click();
    harness.run();

    assert!(shows(&harness, "Step 1 of 16: Audio"));
    assert!(shows(&harness, "Strawberry Player"));
}

#[test]
fn alt_and_the_arrows_move_between_the_steps() {
    let mut harness = first();

    for _ in 0_u8..2 {
        harness.key_press_modifiers(Modifiers::ALT, Key::ArrowRight);
        harness.run();
    }

    assert!(shows(&harness, "Step 2 of 16"));

    harness.key_press_modifiers(Modifiers::ALT, Key::ArrowLeft);
    harness.run();

    assert!(shows(&harness, "Step 1 of 16"));
}

#[test]
fn the_list_jumps_to_the_summary_and_finish_without_picks_shows_success() {
    let mut harness = first();

    harness
        .get_by_role_and_label(Role::ComboBox, "Go to step")
        .click();
    harness.run();
    harness.get_by_label("Summary").click();
    harness.run();

    assert!(shows(&harness, "You picked nothing"));

    harness.get_by_label("Finish").click();
    harness.run();

    assert!(shows(&harness, "Success!"));
    assert_eq!(sent(&harness), [Command::from(HomeCommand::SetupFinished)]);
}

#[test]
fn the_summary_names_the_picks_and_opens_the_review() {
    let mut harness = page(strawberry(), Plans::default(), Stage::Summary);

    assert!(shows(&harness, "Apps to install: 1"));
    assert!(shows(&harness, "Strawberry Player"));

    harness.get_by_label("Review and apply").click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(UiCommand::ReviewChanges)]);
}

#[test]
fn an_apply_that_worked_shows_the_success_page() {
    let plans = Plans {
        finished: 1,
        last_succeeded: true,
        ..Plans::default()
    };

    let harness = page(ChangeSet::default(), plans, Stage::Summary);

    assert!(shows(&harness, "Success!"));
    assert!(shows(&harness, "Finish setup"));
}

#[test]
fn a_test_run_does_not_call_the_system_ready() {
    let plans = Plans {
        finished: 1,
        last_succeeded: true,
        last_ending: Ending::Tested,
    };

    let harness = page(ChangeSet::default(), plans, Stage::Summary);

    assert!(shows(&harness, "Success!"));
    assert!(shows(
        &harness,
        "Test mode: nothing was installed or changed."
    ));
    assert!(!shows(&harness, "ready to go"));
}

#[test]
fn a_run_with_nothing_to_do_does_not_call_the_system_ready() {
    let plans = Plans {
        finished: 1,
        last_succeeded: true,
        last_ending: Ending::Nothing,
    };

    let harness = page(ChangeSet::default(), plans, Stage::Summary);

    assert!(shows(
        &harness,
        "Everything you picked was already in place."
    ));
    assert!(!shows(&harness, "ready to go"));
}

#[test]
fn the_summary_rows_keep_their_padding_from_the_card_edge() {
    let harness = page(strawberry(), Plans::default(), Stage::Summary);
    let edge = harness.get_by_label("Summary").rect().left();
    let name = harness.get_by_label("Strawberry Player").rect().left();
    let padding = toolbox_gui_kit::tokens::current(&harness.ctx)
        .sizes
        .row_padding
        .x;

    assert!(
        name - edge >= padding,
        "the name sits {} px from the card edge",
        name - edge
    );
}

#[test]
fn back_and_next_stay_at_the_bottom_of_the_page() {
    let harness = first();
    let window = harness.ctx.content_rect();
    let next = harness.get_by_label("Next").rect();
    let padding = f32::from(
        toolbox_gui_kit::tokens::current(&harness.ctx)
            .sizes
            .page_padding
            .bottom,
    );

    // Only the key hint's line lies between Next and the page padding.
    let room = window.bottom() - padding - next.bottom();
    assert!(
        (0.0..40.0).contains(&room),
        "Next ends at {} in a page ending at {}",
        next.bottom(),
        window.bottom(),
    );
}

#[test]
fn a_step_names_its_category_once() {
    let mut harness = first();
    harness.get_by_label("Next").click();
    harness.run();

    assert!(shows(&harness, "Step 1 of 16: Audio"));
    let texts: Vec<Role> = harness
        .query_all_by_label("Audio")
        .map(|node| node.accesskit_node().role())
        .filter(|role| matches!(role, Role::Label | Role::Heading))
        .collect();

    assert_eq!(texts, [], "a heading repeats the category");
}

#[test]
fn the_step_list_is_centred_under_the_page_whatever_next_says() {
    let harness = page(strawberry(), Plans::default(), Stage::Summary);
    let list = harness
        .get_by_role_and_label(Role::ComboBox, "Go to step")
        .rect();

    let hint = harness
        .get_by_label("Alt+Right goes on, Alt+Left goes back.")
        .rect();

    assert!(
        (list.center().x - hint.center().x).abs() < 1.0,
        "the list's centre is at {} but the page's at {}",
        list.center().x,
        hint.center().x,
    );
}

#[test]
fn reopening_the_review_after_the_apply_still_leads_to_success() {
    let plans = Plans {
        finished: 1,
        last_succeeded: true,
        ..Plans::default()
    };

    let mut harness = page(strawberry(), plans, Stage::Summary);

    harness.get_by_label("Review and apply").click();
    harness.run();
    harness.state_mut().fixture.changes = ChangeSet::default();
    harness.run();

    assert!(shows(&harness, "Success!"));
}
