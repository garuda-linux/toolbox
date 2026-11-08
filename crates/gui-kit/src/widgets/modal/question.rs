use egui::{Context, Id, Label};

use super::{Confirmation, DialogWidth, ModalWrapper, confirm_buttons};
use crate::widgets::ButtonKind;

/// The texts and look of a [`question`]. Every text is already
/// translated, gui-kit knows no languages.
#[derive(Clone, Copy, Debug)]
pub struct Question<'a> {
    pub title: &'a str,
    pub close: &'a str,
    pub body: &'a str,
    pub cancel: &'a str,
    pub confirm: &'a str,
    pub kind: ButtonKind,
    pub width: DialogWidth,
}

/// What a [`question`] got this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Asked {
    pub confirmed: bool,
    /// The question is over, by either answer, Close, Escape or a click
    /// beside it, so the caller stops asking.
    pub done: bool,
}

pub fn question(ctx: &Context, id: Id, question: Question<'_>) -> Asked {
    let outcome = ModalWrapper::new(id, question.title)
        .title_bar(question.close)
        .width(question.width)
        .show(ctx, |ui| {
            ui.add(Label::new(question.body).wrap());
            confirm_buttons(ui, question.cancel, question.confirm, question.kind)
        });

    Asked {
        confirmed: outcome.inner == Some(Confirmation::Confirmed),
        done: outcome.closed || outcome.inner.is_some(),
    }
}
