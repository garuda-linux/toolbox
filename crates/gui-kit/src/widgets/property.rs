//! Rows of a label and its value, such as a kernel's version or a
//! network's address, after Rerun's property list items. The labels of one
//! list line up, and each value can be copied.

use std::sync::Arc;

use egui::{Align, Galley, Label, Layout, Role, Sense, Ui, WidgetInfo, vec2};

use super::buttons::{SMALL_BUTTON, paint_small_button};
use super::{line, rich};
use crate::fonts::{BODY, Weight};
use crate::toasts::{self, ToastKind};
use crate::{icons, tokens};

/// Between the label column and the values.
const LABEL_GAP: f32 = 16.0;

#[derive(Clone, Copy)]
pub struct PropertyTexts<'a> {
    /// The copy button's name for a row's label, such as "Copy Kernel".
    pub copy: &'a dyn Fn(&str) -> String,
    /// The note a copy raises.
    pub copied: &'a str,
}

/// The rows of one list, gathered before any is drawn so the label column
/// is measured first.
pub struct PropertyList<'a> {
    rows: Vec<Property<'a>>,
}

struct Property<'a> {
    label: &'a str,
    /// What the copy button copies, `None` for a value without text.
    copy: Option<String>,
    value: Value<'a>,
}

enum Value<'a> {
    Text(&'a str),
    Widget(Box<dyn FnOnce(&mut Ui) + 'a>),
}

impl<'a> PropertyList<'a> {
    /// A row whose value is plain text, such as a name or a licence.
    pub fn row(&mut self, label: &'a str, value: &'a str) {
        self.rows.push(Property {
            label,
            copy: None,
            value: Value::Text(value),
        });
    }

    /// A row whose value someone pastes elsewhere, such as a version in a
    /// bug report, with a copy button.
    pub fn row_copied(&mut self, label: &'a str, value: &'a str) {
        self.rows.push(Property {
            label,
            copy: Some(value.to_owned()),
            value: Value::Text(value),
        });
    }

    /// A row whose value is a widget, which the row's label names for
    /// screen readers. `copy` is the text the copy button copies.
    pub fn row_with(
        &mut self,
        label: &'a str,
        copy: Option<&str>,
        value: impl FnOnce(&mut Ui) + 'a,
    ) {
        self.rows.push(Property {
            label,
            copy: copy.map(str::to_owned),
            value: Value::Widget(Box::new(value)),
        });
    }
}

/// The rows `body` adds, their labels in one column as wide as the widest.
pub fn property_list<'a>(
    ui: &mut Ui,
    texts: PropertyTexts<'_>,
    body: impl FnOnce(&mut PropertyList<'a>),
) {
    let mut list = PropertyList { rows: Vec::new() };
    body(&mut list);

    let tokens = tokens::current(ui.ctx());
    let labels: Vec<Arc<Galley>> = list
        .rows
        .iter()
        .map(|row| line(ui, row.label, Weight::Medium, BODY, tokens.secondary))
        .collect();

    let width = labels
        .iter()
        .map(|galley| galley.size().x)
        .fold(0.0_f32, f32::max);

    ui.spacing_mut().item_spacing.y = tokens.sizes.control_gap;
    for row in list.rows {
        show_row(ui, width, row, texts);
    }
}

/// The width a value gets. Only a copied value gives up the copy button's
/// column, a plain one wraps at the list's edge.
fn value_room(ui: &Ui, copied: bool) -> f32 {
    let button = if copied {
        LABEL_GAP + SMALL_BUTTON
    } else {
        0.0
    };

    (ui.available_width() - button).max(0.0)
}

fn show_row(ui: &mut Ui, label_width: f32, row: Property<'_>, texts: PropertyTexts<'_>) {
    let tokens = tokens::current(ui.ctx());
    let Property { label, copy, value } = row;
    let mut copy_button = None;
    let row_rect = ui
        .horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let caption = ui
                .allocate_ui_with_layout(
                    vec2(label_width, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(label_width);
                        ui.add(Label::new(rich(
                            ui,
                            label,
                            Weight::Medium,
                            BODY,
                            tokens.secondary,
                        )))
                    },
                )
                .inner;

            ui.add_space(LABEL_GAP);
            let room = value_room(ui, copy.is_some());
            ui.allocate_ui_with_layout(vec2(room, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_width(room);
                match value {
                    Value::Text(text) => {
                        // A label keeps its text as its value, so with the row's label as
                        // its name a screen reader says both.
                        let shown = ui.add(
                            Label::new(rich(ui, text, Weight::Regular, BODY, tokens.text)).wrap(),
                        );

                        shown.labelled_by(caption.id);
                    }
                    Value::Widget(show) => ui.label_inputs_by(caption.id, show),
                }
            });

            if let Some(text) = copy {
                ui.add_space(LABEL_GAP);
                let alt = (texts.copy)(label);
                let (rect, response) =
                    ui.allocate_exact_size(vec2(SMALL_BUTTON, SMALL_BUTTON), Sense::click());

                response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &alt));

                copy_button = Some((rect, response.on_hover_text(alt), text));
            } else {
                // A row without the button keeps its height, so every row
                // of the list is as tall.
                ui.allocate_exact_size(vec2(0.0, SMALL_BUTTON), Sense::hover());
            }
        })
        .response
        .rect;

    let Some((rect, response, text)) = copy_button else {
        return;
    };

    // Always in the tab order, so a keyboard reaches it, but drawn only for
    // the row under the pointer or the button with the focus.
    if ui.rect_contains_pointer(row_rect) || response.has_focus() {
        let hover = tokens::current(ui.ctx()).hover;
        paint_small_button(ui, rect, &response, icons::EDIT_COPY, hover);
    }

    if response.clicked() {
        ui.ctx().copy_text(text);
        toasts::raise(ui.ctx(), ToastKind::Info, texts.copied);
    }
}
