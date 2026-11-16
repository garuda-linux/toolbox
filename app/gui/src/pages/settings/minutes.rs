//! The minutes field of the update checks.

use egui::{TextEdit, Ui};
use toolbox_config::RefreshInterval;
use toolbox_gui_kit::widgets::{Digits, text_field_with_unit};

/// The minutes being typed and what the last commit was held to.
#[derive(Debug, Default)]
pub struct MinutesField {
    /// The text while it is typed, `None` shows the setting.
    typed: Option<String>,
    /// A typed value was refused and became this: a number outside the
    /// limits, or nothing at all.
    held: Option<u16>,
}

impl MinutesField {
    pub const fn held(&self) -> Option<u16> {
        self.held
    }

    /// The field for `minutes` with `unit` inside it, named `name`.
    /// Returns the interval to save once Enter or leaving the field
    /// commits a number. Only digits can be typed.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        minutes: u16,
        (name, unit): (&str, &str),
        width: f32,
    ) -> Option<RefreshInterval> {
        let mut text = Digits(self.typed.clone().unwrap_or_else(|| minutes.to_string()));

        let response = text_field_with_unit(
            ui,
            TextEdit::singleline(&mut text).desired_width(width),
            unit,
        );

        let label = name.to_owned();
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_label(label));

        if response.changed() {
            self.typed = Some(text.0);
        }

        if !response.lost_focus() {
            return None;
        }

        let typed = self.typed.take()?;
        self.held = None;
        let Ok(number) = typed.trim().parse::<u32>() else {
            self.held = Some(minutes);
            return None;
        };

        let interval = RefreshInterval::clamped(number);
        let RefreshInterval::Minutes(kept) = interval else {
            return None;
        };

        self.held = (u32::from(kept) != number).then_some(kept);

        // The note is drawn from `held` before the field, so the next frame
        // shows it.
        ui.ctx().request_repaint();
        Some(interval)
    }
}
