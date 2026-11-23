//! The border of a form card, which Darkly's cards do without, so it stays
//! for the surfaces that still draw one.

use egui::Color32;

use super::darkly::{darker, frame_fill};
use super::{contrast, mix};

/// `FormCardBackground`'s test of a dark surface: one minus the weighted
/// luma of the colour, at 0.4 or more.
pub fn is_dark_surface(color: Color32) -> bool {
    let (red, green, blue) = (
        f32::from(color.r()) / 255.0,
        f32::from(color.g()) / 255.0,
        f32::from(color.b()) / 255.0,
    );

    let darkness = 1.0 - 0.114_f32.mul_add(blue, 0.299_f32.mul_add(red, 0.587 * green));
    darkness >= 0.4
}

/// The border of a form card, as Kirigami Addons' `FormCardBackground`
/// draws it: a darker View on a dark View, else the View mixed toward the
/// text by 0.15. Its darkness test is the same as the library's.
pub(super) fn form_border(view: Color32, ink: Color32) -> Color32 {
    if is_dark_surface(view) {
        darker(view, 1.2)
    } else {
        mix(view, ink, 0.15)
    }
}

/// Darkly's group box fill (darklyhelper.cpp:180), stepped 4 % toward the
/// text where the View stands within 1.1 of the Window, so a card stays
/// visible on a desktop whose View is its Window colour.
pub(super) fn card_fill(window: Color32, view: Color32, ink: Color32) -> Color32 {
    step_on_flat(frame_fill(window, view), window, view, ink)
}

/// A field's fill, the View, stepped the same way as a card on a flat
/// theme, so a line edit stays apart from the window too.
pub(super) fn input_fill(window: Color32, view: Color32, ink: Color32) -> Color32 {
    step_on_flat(view, window, view, ink)
}

/// `fill` moved 4 % toward the text when the View is within 1.1 of the
/// Window, the only case where the fill would melt into the window.
fn step_on_flat(fill: Color32, window: Color32, view: Color32, ink: Color32) -> Color32 {
    if contrast(view, window) < 1.1 {
        mix(fill, ink, 0.04)
    } else {
        fill
    }
}
