//! Only root may read the ESP, so the helper hands out settings and
//! entries, never a file's text.

mod entry;
mod esp;
mod pe;
mod settings;

use serde::{Deserialize, Serialize};

pub use self::entry::{EntryKind, LoaderEntry};
pub use self::esp::esp_place;
pub use self::pe::{HEADER_BYTES, Section, sections};
pub use self::settings::{LoaderSettings, LoaderTimeout};
use crate::protocol::{DecodeError, decode};

pub const LOADER_CONF: &str = "loader/loader.conf";
pub const UKI_FOLDER: &str = "EFI/Linux";
pub const ENTRIES_FOLDER: &str = "loader/entries";

/// Far more than a real ESP holds, so the answer stays small.
pub const MAX_ENTRIES: usize = 256;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SdBootMenu {
    pub settings: LoaderSettings,
    pub entries: Vec<LoaderEntry>,
}

impl SdBootMenu {
    /// # Errors
    ///
    /// [`DecodeError`] for anything but a menu.
    pub fn parse(line: &str) -> Result<Self, DecodeError> {
        let mut menu: Self = decode(line)?;

        menu.entries.truncate(MAX_ENTRIES);
        Ok(menu)
    }

    pub fn choosable(&self) -> impl Iterator<Item = &LoaderEntry> {
        self.entries.iter().filter(|entry| !entry.is_snapshot())
    }
}
