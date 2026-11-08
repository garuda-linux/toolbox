use egui::Ui;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::rich;
use toolbox_guide::Page;

use super::Jump;
use super::contents::entry;
use crate::i18n;

const SUB_INDENT: f32 = 14.0;

pub(super) fn show(ui: &mut Ui, page: &Page, current: Option<&str>) -> Option<Jump> {
    if page.outline.is_empty() {
        return None;
    }

    let tokens = ui.tokens();
    let title = i18n::text(ui.ctx(), "guide-outline");
    ui.label(rich(ui, &title, Weight::SemiBold, SMALL, tokens.secondary));
    ui.add_space(tokens.sizes.tight_gap);

    let mut jump = None;
    for heading in &page.outline {
        let indent = if heading.level > 2 { SUB_INDENT } else { 0.0 };
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let here = current == Some(heading.anchor.as_str());
            if entry(ui, &heading.text, here).clicked() {
                jump = Some(Jump::Anchor(heading.anchor.clone()));
            }
        });
    }

    jump
}
