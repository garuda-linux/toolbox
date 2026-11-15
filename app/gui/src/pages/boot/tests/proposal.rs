use egui_kittest::kittest::Queryable as _;
use toolbox_plan::PackageName;

use super::{grub_system, menu, page};
use crate::boot::Boot;

/// The answers belong to the proposal, so they sit in its banner at the
/// right, not loose under it.
#[test]
fn the_proposal_answers_sit_in_its_banner() {
    let kernel = PackageName::new("linux-garuda").unwrap();
    let harness = page(grub_system(), Boot::showing(menu(), None, Some(kernel)));

    let said = harness
        .get_by_label_contains("The menu entry of the kernel linux-garuda can become")
        .rect();

    for answer in ["Queue as default", "Do not change"] {
        let button = harness.get_by_label(answer).rect();

        assert!(
            button.left() >= said.right(),
            "{answer} {button:?} {said:?}"
        );
        assert!(
            button.top() < said.bottom(),
            "{answer} sits below the banner"
        );
    }
}
