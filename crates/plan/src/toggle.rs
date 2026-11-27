/// The wire form of a switch that the protocol has always sent as a JSON
/// boolean. Other forms send `"on"` or `"off"`.
pub mod as_bool {
    use serde::{Deserialize as _, Deserializer};

    use super::Toggle;

    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde's `with` passes the field by reference"
    )]
    pub fn serialize<S>(toggle: &Toggle, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(toggle.is_on())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Toggle, D::Error>
    where
        D: Deserializer<'de>,
    {
        bool::deserialize(deserializer).map(Toggle::from_on)
    }
}

/// The wire form of a switch that the protocol has always sent as a JSON
/// boolean or `null` while it is unset. Other forms send `"on"` or `"off"`.
pub mod option_as_bool {
    use serde::{Deserialize as _, Deserializer, Serializer};

    use super::Toggle;

    /// Writes `true`, `false` or `null`.
    ///
    /// # Errors
    ///
    /// The serializer's own error, when it refuses the value.
    pub fn serialize<S>(toggle: &Option<Toggle>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match toggle {
            Some(toggle) => serializer.serialize_some(&toggle.is_on()),
            None => serializer.serialize_none(),
        }
    }

    /// Reads `true`, `false` or `null`.
    ///
    /// # Errors
    ///
    /// The deserializer's own error, when the value is none of those.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Toggle>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<bool>::deserialize(deserializer).map(|on| on.map(Toggle::from_on))
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Toggle {
    On,
    Off,
}

impl Toggle {
    pub const fn from_on(on: bool) -> Self {
        if on { Self::On } else { Self::Off }
    }

    pub const fn is_on(self) -> bool {
        matches!(self, Self::On)
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn flipped(self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}
