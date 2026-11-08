//! Labels are the accessible names Orca reads, so a step finds a control
//! the way a user of a screen reader does.

use egui::accesskit::Role;
use egui::{Key, Modifiers};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    Wait {
        role: Option<Role>,
        label: &'static str,
    },
    WaitContaining(&'static str),
    /// For what differs between machines, such as a page that needs a program one of them lacks.
    WaitEither(&'static str, &'static str),
    WaitGone(&'static str),
    Click {
        role: Option<Role>,
        label: &'static str,
    },
    Text(&'static str),
    Key {
        modifiers: Modifiers,
        key: Key,
    },
    Disabled {
        role: Option<Role>,
        label: &'static str,
    },
    Shot(&'static str),
}

impl Step {
    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn wait(label: &'static str) -> Self {
        Self::Wait { role: None, label }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn click(label: &'static str) -> Self {
        Self::Click { role: None, label }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn tab(label: &'static str) -> Self {
        Self::Click {
            role: Some(Role::Tab),
            label,
        }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn press(key: Key) -> Self {
        Self::Key {
            modifiers: Modifiers::NONE,
            key,
        }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn click_checkbox(label: &'static str) -> Self {
        Self::Click {
            role: Some(Role::CheckBox),
            label,
        }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn button(label: &'static str) -> Self {
        Self::Click {
            role: Some(Role::Button),
            label,
        }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn disabled_button(label: &'static str) -> Self {
        Self::Disabled {
            role: Some(Role::Button),
            label,
        }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn radio(label: &'static str) -> Self {
        Self::Click {
            role: Some(Role::RadioButton),
            label,
        }
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn combo(label: &'static str) -> Self {
        Self::Click {
            role: Some(Role::ComboBox),
            label,
        }
    }

    #[must_use = "the steps do nothing until a suite runs them"]
    pub const fn type_into(field: &'static str, text: &'static str) -> [Self; 2] {
        [
            Self::Click {
                role: Some(Role::TextInput),
                label: field,
            },
            Self::Text(text),
        ]
    }

    #[must_use = "a step does nothing until a suite runs it"]
    pub const fn shortcut(key: Key) -> Self {
        Self::Key {
            modifiers: Modifiers::COMMAND,
            key,
        }
    }
}
