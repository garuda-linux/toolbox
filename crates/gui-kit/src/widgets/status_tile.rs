//! The mark of a status: its plain glyph in the status colour on a soft
//! tile of that colour. Banners, notes, error rows, empty states and
//! status rows draw this one mark, so a status looks the same everywhere.

use egui::{Color32, Pos2, Rect, Response, Role, Sense, Ui, Vec2, vec2};

use super::icon_tile::{TileSize, paint_icon_tile};
use super::mark_word::mark_and_word;
use super::{Tone, line, status_dot};
use crate::fonts::{SMALL, Weight};
use crate::icons::{self, Icon};
use crate::tokens::{self, Tile, Tokens};

impl Tone {
    pub fn status_mark(self, tokens: &Tokens) -> (Tile, Icon) {
        (tokens.status_tile(self.status_colour(tokens)), self.glyph())
    }

    /// Information is the accent, a neutral note the secondary text.
    const fn status_colour(self, tokens: &Tokens) -> Color32 {
        match self {
            Self::Accent => tokens.accent,
            Self::Neutral => tokens.secondary,
            Self::Warning => tokens.warning,
            Self::Success => tokens.success,
            Self::Error => tokens.error,
        }
    }

    const fn glyph(self) -> Icon {
        match self {
            Self::Accent | Self::Neutral => icons::HELP_ABOUT,
            Self::Warning => icons::DIALOG_WARNING,
            Self::Success => icons::DIALOG_POSITIVE,
            Self::Error => icons::DIALOG_ERROR,
        }
    }

    /// The status a state glyph stands for, for callers that name the
    /// glyph alone.
    pub(super) fn of_state_glyph(icon: Icon) -> Option<Self> {
        [Self::Accent, Self::Warning, Self::Success, Self::Error]
            .into_iter()
            .find(|tone| tone.glyph() == icon)
    }
}

pub fn paint_status_tile(ui: &Ui, centre: Pos2, size: TileSize, tone: Tone) {
    let (tile, icon) = tone.status_mark(&tokens::current(ui.ctx()));
    paint_icon_tile(ui, centre, size, icon, tile);
}

pub fn status_tile(ui: &mut Ui, size: TileSize, tone: Tone) -> Rect {
    let edge = size.edge(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(edge, edge), Sense::hover());
    paint_status_tile(ui, rect.center(), size, tone);
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
    let edge = TileSize::Line.edge(ui);
    let gap = tokens.sizes.text_to_icon;
    let text = line(ui, label, Weight::Medium, SMALL, tokens.secondary);
    mark_and_word(
        ui,
        (Vec2::splat(edge), gap),
        text,
        (Role::Label, label),
        |ui, mark| paint_status_tile(ui, mark.center(), TileSize::Line, tone),
    )
}
