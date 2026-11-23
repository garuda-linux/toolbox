use egui::{Rect, Sense, Ui, vec2};

use super::page_layout;
use crate::testing::harness;
use crate::tokens;

const WIDE: f32 = 2400.0;

/// The room the page was given and the rect a full-width block of its
/// body took.
#[derive(Clone, Copy)]
struct Drawn {
    page: Rect,
    block: Rect,
}

/// System Settings and Discover fill the window beside the sidebar, less
/// the page's margins, rather than a narrow centred column.
#[test]
fn a_page_body_has_no_column_cap() {
    let mut harness = harness(WIDE, 300.0).build_ui_state(
        |root, drawn: &mut Drawn| {
            drawn.page = root.available_rect_before_wrap();
            page_layout(
                root,
                "form column",
                drawn,
                |_, _: &mut Ui| false,
                |drawn, ui: &mut Ui| {
                    let size = vec2(ui.available_width(), 20.0);
                    drawn.block = ui.allocate_exact_size(size, Sense::hover()).0;
                },
            );
        },
        Drawn {
            page: Rect::NOTHING,
            block: Rect::NOTHING,
        },
    );

    harness.run();
    let padding = tokens::current(&harness.ctx).sizes.page_padding;
    let Drawn { page, block } = *harness.state();

    assert!(page.width() > WIDE - 50.0, "{page:?}");
    assert!(
        (block.left() - (page.left() + f32::from(padding.left))).abs() < 0.5,
        "{block:?} in {page:?}"
    );
    assert!(
        (block.right() - (page.right() - f32::from(padding.right))).abs() < 0.5,
        "{block:?} in {page:?}"
    );
}
