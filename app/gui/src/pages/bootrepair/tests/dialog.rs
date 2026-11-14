use egui_kittest::kittest::Queryable as _;

use super::{live, page};
use crate::pages::bootrepair::BootRepairPage;

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

/// The question lists what it writes where, the layout included, and
/// below the list its sentences keep one gap between each other.
#[test]
fn the_question_lists_its_facts_above_evenly_spaced_sentences() {
    let mut harness = page(live(), BootRepairPage::default());
    ask_reinstall(&mut harness);

    let layout = harness.get_by_label("Layout").rect();
    let then = harness
        .get_by_label_contains("entered into the firmware's boot menu")
        .rect();

    let runs = harness.get_by_label_contains("It runs right away").rect();

    assert!(layout.bottom() < then.top(), "{layout:?} {then:?}");
    let first = then.top() - layout.bottom();
    let second = runs.top() - then.bottom();
    assert!(
        first >= second - 0.5,
        "{first} px below the list, {second} px between the sentences"
    );
}
