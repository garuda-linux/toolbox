use egui::{Rect, Sense, vec2};
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::tokens;

use super::{Words, row};

#[test]
fn the_control_of_a_long_setting_is_centred_on_the_whole_row() {
    let mut harness = toolbox_gui_kit::testing::harness(700.0, 300.0).build_ui_state(
        |ui, placed: &mut (Rect, Rect)| {
            let words = Words {
                current: None,
                title: "Swap file",
                description: "Moves rarely used memory to the disk, so a full memory slows \
                              the system down rather than closing apps. It takes room on \
                              the disk as large as the memory it can hold.",
                note: Some("Takes effect after a restart."),
                queued: None,
            };

            let top = ui.cursor().top();
            row(ui, true, &words, 160.0, |ui| {
                placed.1 = ui.allocate_exact_size(vec2(160.0, 34.0), Sense::click()).0;
            });

            let drawn = ui.min_rect();
            placed.0 = Rect::from_x_y_ranges(drawn.x_range(), top..=drawn.bottom());
        },
        (Rect::NOTHING, Rect::NOTHING),
    );

    harness.run();
    let title = harness.get_by_label("Swap file").rect();
    let (row, control) = *harness.state();
    let padding = tokens::Sizes::DEFAULT.row_padding.y;
    let content = row.shrink2(vec2(0.0, padding));

    assert!(
        (content.center().y - control.center().y).abs() < 0.5,
        "row content at {content:?}, control at {control:?}"
    );
    assert!(
        control.center().y > title.bottom(),
        "the control sits level with the title at {title:?}: {control:?}"
    );
}
