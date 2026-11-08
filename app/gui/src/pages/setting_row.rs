//! A row of a settings card: the words at the left, the control at the
//! right or below on a narrow window.

use egui::{Label, Sense, Ui, UiBuilder};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{
    Tone, card_row, queued_tag, rich, split_row, status_dot, switch, title_line,
};

/// Room for a switch at a row's right, a card gap apart from the words.
const SWITCH_ROOM: f32 = super::rows::SWITCH_WIDTH + tokens::Sizes::DEFAULT.card_gap;

/// The words of one row: its title, what it does, and a note below when
/// one is due. `queued` is the tag beside the title of a row whose change
/// waits for Apply.
pub struct Words<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub note: Option<&'a str>,
    pub queued: Option<&'a str>,
    /// What the setting holds now, such as a chosen file, as a line under
    /// the description. A long one is cut with "…" and shows whole in a
    /// tooltip.
    pub current: Option<&'a str>,
}

/// A setting whose control is a switch named by the row's title. A click
/// anywhere on the row flips it too, so its target is the whole row.
/// Returns whether it was flipped.
pub fn switch_row(ui: &mut Ui, first: bool, words: &Words<'_>, on: bool, enabled: bool) -> bool {
    let sense = if enabled && ui.is_enabled() {
        Sense::CLICK
    } else {
        Sense::hover()
    };

    // The row senses with its last frame's rect, under the switch, so a
    // click on the switch is the switch's alone.
    ui.scope_builder(UiBuilder::new().sense(sense), |ui| {
        let row_clicked = ui.response().clicked();
        let mut flipped = false;
        row(ui, first, words, SWITCH_ROOM, |ui| {
            flipped = switch(ui, on, words.title, enabled).clicked();
        });
        flipped || row_clicked
    })
    .inner
}

/// One setting: its words at the left, `control`, `width` wide, at the
/// right or below on a narrow window.
pub fn row(ui: &mut Ui, first: bool, words: &Words<'_>, width: f32, control: impl FnOnce(&mut Ui)) {
    stated_row(ui, (first, None), words, width, control);
}

/// A [`row`] with a state, such as "Running", as a dot beside the title,
/// the way the Services page shows a unit's.
pub fn stated_row(
    ui: &mut Ui,
    (first, state): (bool, Option<(&str, Tone)>),
    words: &Words<'_>,
    width: f32,
    control: impl FnOnce(&mut Ui),
) {
    card_row(ui, first, |ui| {
        split_row(
            ui,
            width,
            |ui| {
                let tokens = tokens::current(ui.ctx());
                let title = rich(ui, words.title, Weight::SemiBold, BODY, tokens.text);
                title_line(ui, true, |ui| {
                    ui.label(title);
                    if let Some((state, tone)) = state {
                        status_dot(ui, state, tone);
                    }

                    if let Some(queued) = words.queued {
                        queued_tag(ui, queued, Tone::Accent);
                    }
                });

                if !words.description.is_empty() {
                    let description = rich(
                        ui,
                        words.description,
                        Weight::Regular,
                        SMALL,
                        tokens.secondary,
                    );

                    ui.add(Label::new(description).wrap());
                }

                if let Some(current) = words.current {
                    let shown = rich(ui, current, Weight::Medium, SMALL, tokens.text);
                    ui.add(Label::new(shown).truncate());
                }

                if let Some(note) = words.note {
                    let note = rich(ui, note, Weight::Medium, SMALL, tokens.warning);
                    ui.add(Label::new(note).wrap());
                }
            },
            control,
        );
    });
}

#[cfg(test)]
mod tests;
