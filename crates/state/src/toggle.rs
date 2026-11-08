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
