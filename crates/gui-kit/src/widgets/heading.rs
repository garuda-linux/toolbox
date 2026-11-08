//! The headings of pages and of their sections.

use egui::{Align, Label, Layout, Rect, Ui, UiBuilder, pos2, vec2};

use super::icon_tile::{TileSize, icon_tile};
use super::layout::{MIN_TEXT_WIDTH, SETTLED};
use super::rich;
use crate::fonts::{BODY, SECTION, SMALL, TITLE, Weight};
use crate::icons::Icon;
use crate::tokens;

/// Between a page's title and the controls beside it, as between a
/// row's text and its controls.
const TITLE_GAP: f32 = 16.0;

/// Controls at a heading's right, drawn once it is laid out.
type Actions<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

pub struct SectionHeading<'a> {
    icon: Icon,
    title: &'a str,
    detail: &'a str,
    trailing: Option<(f32, Actions<'a>)>,
}

impl<'a> SectionHeading<'a> {
    #[must_use = "a heading does nothing until shown"]
    pub fn new(icon: Icon, title: &'a str, detail: &'a str) -> Self {
        Self {
            icon,
            title,
            detail,
            trailing: None,
        }
    }

    /// Controls `width` wide at the heading's right, such as a Refresh.
    #[must_use = "a heading does nothing until shown"]
    pub fn trailing(mut self, width: f32, actions: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.trailing = Some((width, Box::new(actions)));
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let tokens = tokens::current(ui.ctx());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            icon_tile(ui, TileSize::Line, self.icon);
            let title = ui.label(rich(ui, self.title, Weight::Bold, SECTION, tokens.text));
            mark_heading(ui, &title, self.title, 2);
            ui.label(rich(
                ui,
                self.detail,
                Weight::Regular,
                SMALL,
                tokens.secondary,
            ));
            if let Some((width, actions)) = self.trailing {
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width().max(width), 0.0),
                    Layout::left_to_right(Align::Center),
                    |ui| super::layout::trailing(ui, actions),
                );
            }
        });
    }
}

/// Tells screen readers that `label` is a heading of `level`, so Orca's
/// heading keys find page, section and dialog titles.
pub(super) fn mark_heading(ui: &Ui, label: &egui::Response, text: &str, level: usize) {
    ui.ctx().accesskit_node_builder(label.id, |node| {
        node.set_role(egui::accesskit::Role::Heading);
        node.set_level(level);
        node.set_label(text);
    });
}

/// A page's title, `trailing` (tabs, refresh) `trailing_width` wide at its
/// right and centred on its line, and the purpose under both to the
/// column's edge, so a longer purpose never moves the controls.
///
/// The controls go under the purpose only when the title would not fit
/// beside them, or not in [`MIN_TEXT_WIDTH`].
pub fn page_header(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    trailing_width: f32,
    trailing: impl FnOnce(&mut Ui),
) {
    let tokens = tokens::current(ui.ctx());
    let gap = tokens.sizes.tight_gap;
    let heading = |column: &mut Ui| {
        let shown = column.label(rich(column, title, Weight::Bold, TITLE, tokens.text));
        mark_heading(column, &shown, title, 1);
        shown.rect
    };

    let purpose = |column: &mut Ui| {
        column.add(
            Label::new(rich(
                column,
                subtitle,
                Weight::Regular,
                BODY,
                tokens.secondary,
            ))
            .wrap(),
        );
    };

    let full = ui.available_width();
    let title_width = one_line_width(ui, title);
    let title_room = (full - trailing_width - TITLE_GAP).max(0.0);
    if trailing_width > 0.0 && title_room < title_width.min(MIN_TEXT_WIDTH) {
        // Stacked under the words, the controls keep their order.
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = gap;
            heading(ui);
            purpose(ui);
            ui.allocate_ui_with_layout(
                vec2(trailing_width.min(full), 0.0),
                Layout::left_to_right(Align::Center),
                trailing,
            );
        });
        return;
    }

    let top_left = ui.cursor().min;
    let drawn =
        ui.allocate_ui_with_layout(vec2(title_room, 0.0), Layout::top_down(Align::Min), |ui| {
            ui.set_width(title_room);
            heading(ui)
        });

    let controls = trailing_slot(ui, drawn.inner, top_left.x + full, trailing_width, trailing);

    // Taller controls push the purpose down rather than cover it.
    let below = drawn.response.rect.bottom().max(controls.bottom()) + gap;
    let words = Rect::from_min_size(pos2(top_left.x, below), vec2(full, f32::INFINITY));
    let bottom = ui
        .scope_builder(UiBuilder::new().max_rect(words), |ui| {
            ui.set_width(full);
            purpose(ui);
        })
        .response
        .rect
        .bottom();

    ui.advance_cursor_after_rect(Rect::from_min_max(
        top_left,
        pos2(top_left.x + full, bottom),
    ));
}

fn one_line_width(ui: &Ui, title: &str) -> f32 {
    let tokens = tokens::current(ui.ctx());
    egui::WidgetText::from(rich(ui, title, Weight::Bold, TITLE, tokens.text))
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        )
        .size()
        .x
}

/// The row under a page's header that stays with it.
///
/// `left` (the search) sits at the column's left edge, `right` (the sort)
/// at its right edge. Only pages with a search or a sort draw one. Both
/// draw from `page`, which they take in turn.
pub fn page_toolbar<P>(
    ui: &mut Ui,
    page: &mut P,
    left: impl FnOnce(&mut P, &mut Ui),
    right: impl FnOnce(&mut P, &mut Ui),
) {
    ui.add_space(tokens::current(ui.ctx()).sizes.card_gap);
    ui.horizontal(|ui| {
        left(page, ui);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| right(page, ui));
    });
}

pub fn section_heading(ui: &mut Ui, icon: Icon, title: &str, detail: &str) {
    SectionHeading::new(icon, title, detail).show(ui);
}

/// The heading of a part inside a section, such as a package's
/// dependencies, with a short `detail` such as their count.
pub fn minor_heading(ui: &mut Ui, title: &str, detail: &str) {
    let tokens = tokens::current(ui.ctx());
    ui.scope(|ui| {
        // As tall as its text, so it starts on one line with text beside
        // it rather than lower in a control's height.
        ui.spacing_mut().interact_size.y = 0.0;
        ui.horizontal(|ui| {
            let shown = ui.label(rich(ui, title, Weight::SemiBold, BODY, tokens.text));
            mark_heading(ui, &shown, title, 3);
            ui.label(rich(ui, detail, Weight::Regular, SMALL, tokens.secondary));
        });
    });
}

/// A section's heading with a sentence on what it holds right under it,
/// then the room before the section's card.
pub fn section_intro(ui: &mut Ui, icon: Icon, title: &str, intro: &str) {
    SectionHeading::new(icon, title, "").show(ui);
    let tokens = tokens::current(ui.ctx());
    ui.add(Label::new(rich(ui, intro, Weight::Regular, BODY, tokens.secondary)).wrap());
    ui.add_space(tokens.sizes.heading_gap);
}

/// The header's controls in reading order, ending at `right` and centred on
/// the `title`'s line. Last frame's size places them, `width` until then.
fn trailing_slot(
    ui: &mut Ui,
    title_rect: Rect,
    right: f32,
    width: f32,
    trailing: impl FnOnce(&mut Ui),
) -> Rect {
    let measured_id = ui.scope_id().with("header controls size");
    let measured = ui
        .data(|data| data.get_temp::<egui::Vec2>(measured_id))
        .unwrap_or_else(|| vec2(width, 0.0));

    let height = measured.y.max(title_rect.height());
    let slot = Rect::from_center_size(
        pos2(right - measured.x / 2.0, title_rect.center().y),
        vec2(measured.x, height),
    );

    let controls = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(slot)
                .layout(Layout::left_to_right(Align::Center)),
            trailing,
        )
        .response
        .rect;

    if (controls.size() - measured).length() > SETTLED {
        ui.data_mut(|data| data.insert_temp(measured_id, controls.size()));
        ui.ctx()
            .request_discard("the header controls changed their size");
    }

    controls
}
