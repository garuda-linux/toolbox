//! A password field with a button that shows what was typed, so a long
//! password can be checked before it is sent.

use egui::{Align2, Id, Response, TextEdit, Ui};

use super::SecretText;
use super::field::{FieldButton, Parts, field};
use crate::icons;

/// The words of a [`secret_field`]. `name` names the field for screen
/// readers, `show` and `hide` its button in each state.
#[derive(Clone, Copy, Debug)]
pub struct SecretTexts<'a> {
    pub name: &'a str,
    pub show: &'a str,
    pub hide: &'a str,
}

/// A field for `text` as wide as its room, in dots until its button shows
/// the text. [`super::forget_text`] with the field's id hides it again.
pub fn secret_field(ui: &mut Ui, text: &mut SecretText, texts: SecretTexts<'_>) -> Response {
    let id = ui.make_persistent_id(("secret_field", texts.name));
    let flag = revealed_id(id);
    let revealed = ui.data(|data| data.get_temp::<bool>(flag)) == Some(true);
    let button = if revealed {
        FieldButton {
            icon: icons::VIEW_HIDDEN,
            name: texts.hide,
        }
    } else {
        FieldButton {
            icon: icons::VIEW_VISIBLE,
            name: texts.show,
        }
    };

    let edit = TextEdit::singleline(text)
        .id(id)
        .password(!revealed)
        .desired_width(f32::INFINITY)
        .align(Align2::LEFT_CENTER);

    let parts = Parts {
        button: Some(button),
        ..Parts::default()
    };

    let (response, toggled) = field(ui, edit, parts);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(texts.name));

    if toggled {
        ui.data_mut(|data| data.insert_temp(flag, !revealed));
    }

    response
}

/// Where egui keeps whether the field `field` shows its text.
pub(super) fn revealed_id(field: Id) -> Id {
    field.with("revealed")
}
