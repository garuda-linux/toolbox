use egui::Theme;
use toolbox_gui::Setup;
use toolbox_gui_kit::testing::repaints_once_settled;

use super::{WIDE, page, pages};

/// Packages never loads in the coverage setup. Its skeleton counts as at
/// rest, since test harnesses stand the shimmer still.
#[test]
fn every_settled_page_asks_for_no_frames() {
    let busy: Vec<_> = pages()
        .filter_map(|id| {
            let mut harness = page(id, Theme::Dark, WIDE, Setup::default());
            let repaints = repaints_once_settled(&mut harness);
            (!repaints.is_empty()).then_some((id, repaints))
        })
        .collect();

    assert!(busy.is_empty(), "{busy:#?}");
}
