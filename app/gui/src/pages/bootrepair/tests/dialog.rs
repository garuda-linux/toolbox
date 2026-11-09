use egui_kittest::kittest::Queryable as _;

use super::{live, page};
use crate::pages::bootrepair::BootRepairPage;

/// The question's paragraphs keep one gap between each other, the place
/// the system is looked for included.
#[test]
fn the_questions_paragraphs_are_evenly_apart() {
    let mut harness = page(live(), BootRepairPage::default());
    harness
        .get_by_label("Reinstall the boot loader…")
        .scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall the boot loader…").click();
    harness.run();

    let looked = harness
        .get_by_label_contains("The system is looked for here")
        .rect();

    // The page names the repair too, the paragraph above is the question's.
    let does = harness
        .get_all_by_label_contains("The boot loader is written")
        .map(|node| node.rect())
        .filter(|rect| rect.bottom() <= looked.top())
        .max_by(|one, other| one.bottom().total_cmp(&other.bottom()))
        .expect("the question says what it writes");

    let runs = harness.get_by_label_contains("It runs right away").rect();

    let first = looked.top() - does.bottom();
    let second = runs.top() - looked.bottom();
    assert!(
        (first - second).abs() < 0.5,
        "{first} px before the layout, {second} px after it"
    );
}

fn ask_reinstall(harness: &mut super::Shown) {
    harness
        .get_by_label("Reinstall the boot loader…")
        .scroll_to_me();
    harness.run();
    harness.get_by_label("Reinstall the boot loader…").click();
    harness.run();
}

/// On an installed system the disk is not the live ISO's, so the question
/// says whose programs run as root.
#[test]
fn outside_the_live_iso_the_question_names_the_trust_in_the_disk() {
    use toolbox_core::features::bootrepair::page::{Offer, Progress};

    use crate::bootrepair::BootRepair;

    let mut installed = page(
        BootRepair::showing(Offer::Test, super::sample(), Progress::None),
        BootRepairPage::default(),
    );

    ask_reinstall(&mut installed);

    assert!(installed.query_by_label_contains("run as root").is_some());

    let mut on_the_iso = page(live(), BootRepairPage::default());
    ask_reinstall(&mut on_the_iso);

    assert!(on_the_iso.query_by_label_contains("run as root").is_none());
}
