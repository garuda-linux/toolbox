use egui::{Label, Rect, Ui};

use super::super::{
    ButtonKind, card, card_row, pill_button, rich, split_row, title_line, title_line_height,
};
use crate::fonts::{BODY, SMALL, Weight};
use crate::testing::harness;
use crate::tokens;

/// The Ethernet card of a real laptop as `lspci -nn` names it, too long
/// for one line of a narrow row.
const NAME: &str = "Realtek Semiconductor Co., Ltd. RTL8111/8168/8211/8411 \
                    PCI Express Gigabit Ethernet Controller [10ec:8168] (rev 15)";

/// Where the title's label and the kind under it went, and how tall one
/// title line is.
#[derive(Clone, Copy)]
struct Placed {
    title: Rect,
    kind: Rect,
    line: f32,
}

fn words(ui: &mut Ui) -> (Rect, Rect) {
    let tokens = tokens::current(ui.ctx());
    let title = rich(ui, NAME, Weight::SemiBold, BODY, tokens.text);
    let title = title_line(ui, true, |ui| ui.label(title).rect);
    let kind = rich(ui, "Ethernet", Weight::Regular, SMALL, tokens.secondary);
    (title, ui.add(Label::new(kind).wrap()).rect)
}

/// The row in a window `width` wide with a button at its right.
fn row(width: f32) -> Placed {
    let mut harness = harness(width, 300.0).build_ui_state(
        |ui, placed: &mut Placed| {
            card(ui, |ui| {
                card_row(ui, true, |ui| {
                    placed.line = title_line_height(ui);
                    let ((title, kind), ()) = split_row(ui, 150.0, words, |ui| {
                        pill_button(ui, "Copy this line", ButtonKind::Soft);
                    });

                    placed.title = title;
                    placed.kind = kind;
                });
            });
        },
        Placed {
            title: Rect::NOTHING,
            kind: Rect::NOTHING,
            line: 0.0,
        },
    );

    harness.run();
    *harness.state()
}

#[test]
fn a_wrapped_title_keeps_the_line_under_it_clear() {
    for width in [420.0, 520.0, 640.0] {
        let placed = row(width);
        let gap = placed.kind.top() - placed.title.bottom();
        let tight = tokens::Sizes::DEFAULT.tight_gap;

        assert!(
            placed.title.height() > placed.line + 0.5,
            "width {width}: the title {:?} fits one line",
            placed.title
        );
        assert!(
            (gap - tight).abs() <= 0.5,
            "width {width}: gap {gap}, title {:?}, kind {:?}",
            placed.title,
            placed.kind
        );
    }
}
