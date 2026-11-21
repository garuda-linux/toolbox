//! Code in the guide, in the colours the Console prints in
//! ([`tokens::CodeInks`]).

#[cfg(test)]
mod tests;

use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat, TextStyle, Ui};
use egui_extras::syntax_highlighting::{CodeTheme, TokenType, highlight};

use crate::tokens;

/// A code block's text built part by part, one [`LayoutJob`] for one
/// label, so selection and copy take the text as it was written.
pub struct CodeJob {
    job: LayoutJob,
    theme: CodeTheme,
    inks: tokens::CodeInks,
    font: FontId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Plain,
    Keyword,
    Function,
    String,
    Dim,
}

impl CodeJob {
    pub fn new(ui: &Ui) -> Self {
        let inks = tokens::current(ui.ctx()).code;
        let mut theme = CodeTheme::from_style(ui.style());
        let kinds = [
            (TokenType::Comment, inks.dim),
            (TokenType::Function, inks.function),
            (TokenType::Keyword, inks.keyword),
            (TokenType::Literal, inks.plain),
            (TokenType::StringLiteral, inks.string),
            (TokenType::Punctuation, inks.plain),
            (TokenType::Whitespace, Color32::TRANSPARENT),
        ];

        for (kind, ink) in kinds {
            theme.format_mut(kind).color = ink;
        }

        Self {
            job: LayoutJob::default(),
            theme,
            inks,
            font: TextStyle::Monospace.resolve(ui.style()),
        }
    }

    pub fn plain(&mut self, text: &str) {
        self.inked(text, Ink::Plain);
    }

    pub fn inked(&mut self, text: &str, ink: Ink) {
        let color = match ink {
            Ink::Plain => self.inks.plain,
            Ink::Keyword => self.inks.keyword,
            Ink::Function => self.inks.function,
            Ink::String => self.inks.string,
            Ink::Dim => self.inks.dim,
        };

        self.job
            .append(text, 0.0, TextFormat::simple(self.font.clone(), color));
    }

    pub fn highlighted(&mut self, ui: &Ui, text: &str, language: &str) {
        let part = highlight(ui.ctx(), ui.style(), &self.theme, text, language);
        for section in &part.sections {
            let range = section.byte_range.start.0..section.byte_range.end.0;
            if let Some(piece) = part.text.get(range) {
                self.job.append(piece, 0.0, section.format.clone());
            }
        }
    }

    #[must_use = "the job is what a label shows"]
    pub fn finish(self) -> LayoutJob {
        self.job
    }
}
