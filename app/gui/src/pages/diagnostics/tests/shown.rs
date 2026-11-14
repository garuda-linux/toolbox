//! The report box names the report it holds, which the picker may no
//! longer show.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::{page, report};

#[test]
fn the_report_box_names_its_report_while_the_picker_shows_another() {
    let harness = page(report());

    let picker = harness.get_by_role_and_label(Role::ComboBox, "Report to collect");
    assert_eq!(
        picker.accesskit_node().value().as_deref(),
        Some("System information")
    );

    let caption = harness.get_by_label("Kernel log").rect();
    let line = harness.get_by_label("kernel: hello").rect();
    assert!(
        caption.bottom() <= line.top(),
        "the caption {caption:?} stands above the report {line:?}"
    );
}
