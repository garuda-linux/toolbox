//! The headings of pages and of their sections.

mod toolbar;

use egui::{Align, Label, Layout, Rect, Sense, Ui, UiBuilder, pos2, vec2};
pub use toolbar::{page_toolbar, toolbar_end_inset};
use toolbar::{toolbar_room_id, trailing_gap};

use super::layout::{MIN_TEXT_WIDTH, Redraw, remember};
use super::page::stash_purpose;
use super::rich;
use crate::fonts::{BODY, PAGE_TITLE, SMALL, Weight};
use crate::tokens;

/// The least width of a page's search field, as its room shrinks to. It
/// holds the glyph, the hint and the clear button of the field.
pub const SEARCH_LEAST: f32 = 180.0;

/// Controls at a heading's right, drawn once it is laid out.
type Actions<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

/// A control's width in the row and what it draws.
type Control<'a> = (f32, Actions<'a>);

/// The measures of a title row, as [`PageHeader::show`] works them out.
struct Row<'a> {
    title: &'a str,
    full: f32,
    /// The leading control and its gap to the title, zero without one.
    lead: f32,
    leading_width: f32,
    title_width: f32,
    title_room: f32,
    trailing_width: f32,
    /// The room between the toolbar and the trailing controls' rects.
    trailing_gap: f32,
}

pub struct SectionHeading<'a> {
    title: &'a str,
    detail: &'a str,
    trailing: Option<Control<'a>>,
}

impl<'a> SectionHeading<'a> {
    #[must_use = "a heading does nothing until shown"]
    pub fn new(title: &'a str, detail: &'a str) -> Self {
        Self {
            title,
            detail,
            trailing: None,
        }
    }

    /// Controls `width` wide at the heading's right, such as a Refresh.
    #[must_use = "a heading does nothing until shown"]
    pub fn trailing(mut self, width: f32, actions: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.trailing = Some(control(width, actions));
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let tokens = tokens::current(ui.ctx());
        ui.horizontal(|ui| {
            // `FormHeader` is DemiBold. Bold stands in, as SemiBold draws in the regular face.
            let title = ui.label(rich(ui, self.title, Weight::Bold, BODY, tokens.text));
            mark_heading(ui, &title, self.title, 2);
            if !self.detail.is_empty() {
                ui.label(rich(
                    ui,
                    self.detail,
                    Weight::Regular,
                    SMALL,
                    tokens.secondary,
                ));
            }

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

/// A page's toolbar row: its title, its purpose and the controls at either
/// end. The row is `toolbar_height` tall and centres its title and controls.
///
/// The purpose is drawn at the top of the body ([`stash_purpose`]), so it
/// scrolls with the page. The controls go under the title only when the
/// title would not fit beside them, or not in [`MIN_TEXT_WIDTH`].
#[must_use = "a header does nothing until shown"]
pub struct PageHeader<'a> {
    title: &'a str,
    purpose: &'a str,
    leading: Option<Control<'a>>,
    trailing: Option<Control<'a>>,
    trailing_inset: f32,
}

impl<'a> PageHeader<'a> {
    #[must_use = "a header does nothing until shown"]
    pub const fn new(title: &'a str, purpose: &'a str) -> Self {
        Self {
            title,
            purpose,
            leading: None,
            trailing: None,
            trailing_inset: 0.0,
        }
    }

    /// How far inside its rect the first trailing control paints, so the
    /// toolbar stands one group gap from what both paint. Unset, the gap
    /// is taken between the rects.
    #[must_use = "a header does nothing until shown"]
    pub const fn trailing_inset(mut self, inset: f32) -> Self {
        self.trailing_inset = inset;
        self
    }

    /// A control `width` wide at the row's left, before the title, as
    /// Kirigami's toolbar puts its back button.
    #[must_use = "a header does nothing until shown"]
    pub fn leading(mut self, width: f32, actions: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.leading = Some(control(width, actions));
        self
    }

    /// Controls `width` wide at the row's right, such as tabs and Refresh.
    #[must_use = "a header does nothing until shown"]
    pub fn trailing(mut self, width: f32, actions: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.trailing = Some(control(width, actions));
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let tokens = tokens::current(ui.ctx());
        let (leading_width, leading) = unwrap_control(self.leading);
        let (trailing_width, trailing) = unwrap_control(self.trailing);
        let lead = if leading_width > 0.0 {
            leading_width + tokens.sizes.text_to_control
        } else {
            0.0
        };

        stash_purpose(ui.ctx(), self.purpose);

        let full = ui.available_width();
        let title_width = one_line_width(ui, self.title);
        let title_room = (full - lead - trailing_width - tokens.sizes.text_to_control).max(0.0);
        let row = Row {
            title: self.title,
            full,
            lead,
            leading_width,
            title_width,
            title_room,
            trailing_width,
            trailing_gap: trailing_gap(
                trailing_width,
                self.trailing_inset,
                tokens.sizes.text_to_control,
            ),
        };

        if trailing_width > 0.0 && title_room < title_width.min(MIN_TEXT_WIDTH) {
            stacked(ui, &row, leading, trailing);
        } else {
            beside(ui, &row, leading, trailing);
        }
    }
}

fn control<'a>(width: f32, actions: impl FnOnce(&mut Ui) + 'a) -> Control<'a> {
    (width, Box::new(actions))
}

/// A control's width and actions, or no width and nothing to draw.
fn unwrap_control(control: Option<Control<'_>>) -> Control<'_> {
    control.unwrap_or_else(|| (0.0, Box::new(|_: &mut Ui| {})))
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

/// A page's title on the toolbar row with `trailing` (tabs, refresh) at its
/// right. See [`PageHeader`].
pub fn page_header<'a>(
    ui: &mut Ui,
    title: &'a str,
    subtitle: &'a str,
    trailing_width: f32,
    trailing: impl FnOnce(&mut Ui) + 'a,
) {
    PageHeader::new(title, subtitle)
        .trailing(trailing_width, trailing)
        .show(ui);
}

/// The controls stacked under the title, in their order.
fn stacked(
    ui: &mut Ui,
    row: &Row<'_>,
    leading: impl FnOnce(&mut Ui),
    trailing: impl FnOnce(&mut Ui),
) {
    let tokens = tokens::current(ui.ctx());
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
        ui.allocate_ui_with_layout(
            vec2(row.full, tokens.sizes.toolbar_height),
            Layout::left_to_right(Align::Center),
            |ui| {
                if row.lead > 0.0 {
                    leading(ui);
                    ui.add_space(tokens.sizes.text_to_control);
                }

                title_heading(ui, row.title);
            },
        );
        ui.allocate_ui_with_layout(
            vec2(row.trailing_width.min(row.full), 0.0),
            Layout::left_to_right(Align::Center),
            trailing,
        );
    });
}

/// The title and its controls on one row, the title's free room kept for
/// the search and sort that come after it.
fn beside(
    ui: &mut Ui,
    row: &Row<'_>,
    leading: impl FnOnce(&mut Ui),
    trailing: impl FnOnce(&mut Ui),
) {
    let tokens = tokens::current(ui.ctx());
    let row_height = tokens.sizes.toolbar_height;
    let (bar, _) = ui.allocate_exact_size(vec2(row.full, row_height), Sense::hover());
    if row.lead > 0.0 {
        // Laid out from the row's top, so the caller centres what needs it.
        let at = Rect::from_min_size(bar.min, vec2(row.leading_width, row_height));
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(at)
                .layout(Layout::left_to_right(Align::Min)),
            leading,
        );
    }

    let gap = tokens.sizes.text_to_control;
    let title_rect = Rect::from_min_size(
        pos2(bar.left() + row.lead, bar.top()),
        vec2(row.title_room, row_height),
    );

    let free = Rect::from_min_max(
        pos2(
            bar.left() + row.lead + row.title_width.min(row.title_room) + gap,
            bar.top(),
        ),
        pos2(
            bar.right() - row.trailing_width - row.trailing_gap,
            bar.bottom(),
        ),
    );

    ui.ctx()
        .data_mut(|data| data.insert_temp(toolbar_room_id(), free));
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(title_rect)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| title_heading(ui, row.title),
    );

    trailing_slot(ui, bar, bar.right(), row.trailing_width, trailing);

    // The title's own scope moves the cursor up to its words, so the row
    // is passed again for what comes below it.
    ui.advance_cursor_after_rect(bar);
}

fn title_heading(column: &mut Ui, title: &str) {
    let tokens = tokens::current(column.ctx());
    let shown = column.add(
        Label::new(rich(
            column,
            title,
            Weight::Regular,
            PAGE_TITLE,
            tokens.text,
        ))
        .truncate(),
    );

    mark_heading(column, &shown, title, 1);
}

fn one_line_width(ui: &Ui, title: &str) -> f32 {
    let tokens = tokens::current(ui.ctx());
    egui::WidgetText::from(rich(ui, title, Weight::Regular, PAGE_TITLE, tokens.text))
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        )
        .size()
        .x
}

pub fn section_heading(ui: &mut Ui, title: &str, detail: &str) {
    SectionHeading::new(title, detail).show(ui);
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
pub fn section_intro(ui: &mut Ui, title: &str, intro: &str) {
    SectionHeading::new(title, "").show(ui);
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
) {
    let measured_id = ui.scope_id().with("header controls size");
    let measured = ui
        .data(|data| data.get_temp::<egui::Vec2>(measured_id))
        .unwrap_or_else(|| vec2(width, 0.0));

    let height = measured.y.max(title_rect.height());
    let slot = Rect::from_center_size(
        pos2(right - measured.x / 2.0, title_rect.center().y),
        vec2(measured.x, height),
    );

    // The kept size is the last header's, also another page's. A control that
    // fills its room would stay that narrow, so the room is at least the
    // width asked for.
    let room = slot.with_max_x(slot.left() + measured.x.max(width));
    let controls = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(room)
                .layout(Layout::left_to_right(Align::Center)),
            trailing,
        )
        .response
        .rect;

    let reason = "the header controls changed their size";
    remember(
        ui,
        measured_id,
        Some(measured),
        controls.size(),
        (Redraw::Discards, reason),
    );
}
