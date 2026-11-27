//! What the global menu shows, as plain values the app builds from its
//! state.

/// The menus of the bar, left to right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu<A> {
    pub menus: Vec<Submenu<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submenu<A> {
    pub label: String,
    pub entries: Vec<Entry<A>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry<A> {
    Item(Item<A>),
    Separator,
}

/// A line of a menu. A click hands `action` back to the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<A> {
    pub action: A,
    pub label: String,
    pub enabled: bool,
    pub toggle: Toggle,
    pub shortcut: Option<Shortcut>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    None,
    Check(bool),
    Radio(bool),
}

/// A key with the modifiers held for it. `key` is named as Qt names it
/// (`Q`, `F1`, `plus`), which Plasma's menu reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub key: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    Control,
    Alt,
    Shift,
    Super,
}

impl<A> Item<A> {
    /// An enabled item without a check mark or shortcut.
    pub const fn new(action: A, label: String) -> Self {
        Self {
            action,
            label,
            enabled: true,
            toggle: Toggle::None,
            shortcut: None,
        }
    }
}

impl Toggle {
    pub(crate) const fn kind(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Check(_) => Some("checkmark"),
            Self::Radio(_) => Some("radio"),
        }
    }

    pub(crate) const fn state(self) -> Option<i32> {
        match self {
            Self::None => None,
            Self::Check(on) | Self::Radio(on) => Some(if on { 1 } else { 0 }),
        }
    }
}

impl Shortcut {
    pub(crate) fn chords(&self) -> Vec<Vec<String>> {
        let names = self.modifiers.iter().map(|modifier| modifier.name());
        vec![names.chain([self.key]).map(String::from).collect()]
    }
}

impl Modifier {
    const fn name(self) -> &'static str {
        match self {
            Self::Control => "Control",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
            Self::Super => "Super",
        }
    }
}
