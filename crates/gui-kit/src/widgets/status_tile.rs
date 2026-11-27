//! The mark of a status: Breeze's colour status icon, drawn plain at
//! the icon size, as KDE's inline messages draw theirs. Banners, notes,
//! error rows, empty states and status rows draw this one mark, so a
//! status looks the same everywhere.

use egui::{Color32, Pos2, Rect, Response, Role, Sense, Ui, Vec2, vec2};

use super::icon_tile::{ICON_EDGE, paint_icon};
use super::mark_word::mark_and_word;
use super::{Tone, line, status_dot};
use crate::fonts::{SMALL, Weight};
use crate::icons::{Icon, colour};
#[cfg(test)]
use crate::tokens::Tile;
use crate::tokens::{self, Tokens};

impl Tone {
    #[cfg(test)]
    pub fn status_mark(self, tokens: &Tokens) -> (Tile, Icon) {
        (tokens.status_tile(self.status_colour(tokens)), self.glyph())
    }

    /// Information is the accent, a neutral note the secondary text.
    pub(super) const fn status_colour(self, tokens: &Tokens) -> Color32 {
        match self {
            Self::Accent => tokens.accent,
            Self::Neutral => tokens.secondary,
            Self::Warning => tokens.warning,
            Self::Success => tokens.success,
            Self::Error => tokens.error,
        }
    }

    /// Information and a neutral note both take KDE's information icon,
    /// as KDE has no neutral one.
    const fn glyph(self) -> Icon {
        match self {
            Self::Accent | Self::Neutral => colour::DIALOG_INFORMATION,
            Self::Warning => colour::DIALOG_WARNING,
            Self::Success => colour::DIALOG_POSITIVE,
            Self::Error => colour::DIALOG_ERROR,
        }
    }
}

pub fn paint_status_tile(ui: &Ui, centre: Pos2, tone: Tone) {
    paint_icon(ui, centre, tone.glyph());
}

pub fn status_tile(ui: &mut Ui, tone: Tone) -> Rect {
    let edge = ICON_EDGE;
    let (rect, _) = ui.allocate_exact_size(vec2(edge, edge), Sense::hover());
    paint_status_tile(ui, rect.center(), tone);
    rect
}

/// A state that changes, such as the Console's, read when it changes.
///
/// An ended state (success, warning, error) shows its mark, a running or
/// idle one a [`status_dot`], then `label` in the secondary colour. A
/// failure is read at once.
pub fn status_word(ui: &mut Ui, label: &str, tone: Tone) -> Response {
    let response = match tone {
        Tone::Accent | Tone::Neutral => status_dot(ui, label, tone),
        Tone::Warning | Tone::Success | Tone::Error => marked_word(ui, label, tone),
    };

    super::banner::announce(ui, &response, tone, label);
    response
}

fn marked_word(ui: &mut Ui, label: &str, tone: Tone) -> Response {
    let tokens = tokens::current(ui.ctx());
    let edge = ICON_EDGE;
    let gap = tokens.sizes.text_to_icon;
    let text = line(ui, label, Weight::Medium, SMALL, tokens.secondary);
    mark_and_word(
        ui,
        (Vec2::splat(edge), gap),
        text,
        (Role::Label, label),
        |ui, mark| paint_status_tile(ui, mark.center(), tone),
    )
}
