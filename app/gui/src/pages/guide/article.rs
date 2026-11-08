use egui::{Align, Layout, Rect, ScrollArea, Ui, UiBuilder};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, named_pill_button, rich, text_link};
use toolbox_guide::{Guide, Page};

use super::markdown::{self, Reader};
use super::{GuidePage, Jump};
use crate::command::UiCommand;
use crate::i18n;
use crate::page::PageId;
use crate::pages::PageContext;

/// The widest the text of a page runs, so lines stay readable.
const TEXT_WIDTH: f32 = 760.0;

#[derive(Debug, Default)]
pub(super) struct Shown {
    pub jump: Option<Jump>,
    pub reached: bool,
    pub current: Option<String>,
}

pub(super) fn show(
    ui: &mut Ui,
    rect: Rect,
    guide: &Guide,
    state: &mut GuidePage,
    context: &PageContext<'_>,
) -> Shown {
    let Some(page) = guide.page(&state.file) else {
        return Shown::default();
    };

    let mut area = ScrollArea::vertical()
        .id_salt(("guide-article", &state.file))
        .auto_shrink(false);

    if state.fresh {
        area = area.vertical_scroll_offset(0.0);
        state.fresh = false;
    }

    let mut reader = Reader::new(state.target.as_deref());
    let mut inner = ui.new_child(UiBuilder::new().max_rect(rect));
    let view_top = rect.top();
    area.show(&mut inner, |column| {
        column.set_width(column.available_width());
        column.set_max_width(TEXT_WIDTH.min(column.available_width()));
        top_line(column, page, context);
        markdown::blocks(column, &page.blocks, &mut reader);
        neighbours(column, guide, page, &mut reader);
        column.add_space(column.tokens().sizes.section_gap);
    });

    let current = reader
        .headings
        .iter()
        .take_while(|(_, top)| *top <= view_top + 24.0)
        .last()
        .or_else(|| reader.headings.first())
        .map(|(anchor, _)| anchor.clone());

    Shown {
        jump: reader.jump,
        reached: reader.reached,
        current,
    }
}

fn top_line(ui: &mut Ui, page: &Page, context: &PageContext<'_>) {
    let tokens = ui.tokens();
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        let trail = page.part.as_ref().map_or_else(
            || page.title.clone(),
            |part| format!("{part} › {}", page.title),
        );

        ui.label(rich(ui, &trail, Weight::Regular, SMALL, tokens.secondary));

        if let Some(target) = PageId::of_guide_file(&page.file) {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let name = i18n::text(&ctx, target.text_id());
                let label = i18n::text_with(&ctx, "guide-open-page", &[("page", &name)]);
                if named_pill_button(ui, &label, ButtonKind::Soft, &label).clicked() {
                    context.commands.send(UiCommand::Navigate(target));
                }
            });
        }
    });
    ui.add_space(tokens.sizes.card_gap);
}

fn neighbours(ui: &mut Ui, guide: &Guide, page: &Page, reader: &mut Reader<'_>) {
    let ctx = ui.ctx().clone();
    ui.add_space(ui.tokens().sizes.section_gap);
    ui.separator();
    ui.horizontal(|ui| {
        if let Some(before) = guide.previous(&page.file) {
            let text = i18n::text_with(&ctx, "guide-previous", &[("page", &before.title)]);
            if text_link(ui, &text, &text).clicked() {
                reader.jump = Some(Jump::Page {
                    file: before.file.clone(),
                    anchor: None,
                });
            }
        }

        if let Some(after) = guide.next(&page.file) {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let text = i18n::text_with(&ctx, "guide-next", &[("page", &after.title)]);
                if text_link(ui, &text, &text).clicked() {
                    reader.jump = Some(Jump::Page {
                        file: after.file.clone(),
                        anchor: None,
                    });
                }
            });
        }
    });
}
