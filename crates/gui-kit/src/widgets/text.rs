use std::sync::Arc;

use egui::{Color32, Galley, RichText, TextStyle, TextWrapMode, Ui, WidgetText};

use crate::fonts::{self, Weight};

#[must_use = "the text does nothing until added to a ui"]
pub fn rich(ui: &Ui, text: &str, weight: Weight, scale: f32, color: Color32) -> RichText {
    RichText::new(text)
        .font(fonts::font(ui.style(), weight, scale))
        .color(color)
}

#[must_use = "the galley does nothing until painted"]
pub fn line(ui: &Ui, text: &str, weight: Weight, scale: f32, color: Color32) -> Arc<Galley> {
    WidgetText::from(rich(ui, text, weight, scale, color)).into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Body,
    )
}

pub(super) fn cut(ui: &Ui, text: RichText, width: f32) -> Arc<Galley> {
    WidgetText::from(text).into_galley(ui, Some(TextWrapMode::Truncate), width, TextStyle::Body)
}
