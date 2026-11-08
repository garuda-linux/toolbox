use egui::epaint::{RectShape, Shadow};
use egui::{Color32, CornerRadius, Frame, InnerResponse, Margin, Rect, Shape, Stroke, Ui, vec2};

use crate::tokens;

#[derive(Clone, Copy, Debug)]
pub struct Glass {
    frame: Frame,
    tint: Color32,
}

impl Glass {
    #[must_use = "a glass does nothing until shown"]
    pub const fn new(frame: Frame, tint: Color32) -> Self {
        Self { frame, tint }
    }

    #[must_use = "a glass does nothing until shown"]
    pub fn raised(ui: &Ui, corners: CornerRadius, margin: Margin) -> Self {
        let frame = Frame::new()
            .stroke(Stroke::new(1.0, tokens::current(ui.ctx()).outline))
            .corner_radius(corners)
            .inner_margin(margin)
            .shadow(ui.visuals().popup_shadow);

        Self::new(frame, super::raised(ui.ctx()))
    }

    /// The frame to hand to a container that draws it itself, such as a
    /// popup. Its content then goes through [`Glass::inside`].
    pub fn frame(self) -> Frame {
        self.frame.fill(Color32::TRANSPARENT).shadow(Shadow::NONE)
    }

    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.frame().show(ui, |ui| self.inside(ui, body))
    }

    /// For the inside of [`Glass::frame`].
    pub fn inside<R>(self, ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
        let shadow = ui.painter().add(Shape::Noop);
        let blur = super::reserve(ui.painter());
        let slot = ui.painter().add(Shape::Noop);
        let inner = body(ui);

        let margin = self.frame.inner_margin + self.frame.outer_margin;
        let content = ui.min_rect();
        let rect = Rect::from_min_max(
            content.min - vec2(f32::from(margin.left), f32::from(margin.top)),
            content.max + vec2(f32::from(margin.right), f32::from(margin.bottom)),
        );

        let corners = self.frame.corner_radius;
        ui.painter()
            .set(shadow, self.frame.shadow.as_shape(rect, corners));
        if let Some(blur) = blur {
            blur.fill(ui.ctx(), rect, corners);
        }

        ui.painter()
            .set(slot, RectShape::filled(rect, corners, self.tint));
        inner
    }
}
