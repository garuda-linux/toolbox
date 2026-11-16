//! Root-owned config files the Config Files page edits: which paths the
//! helper may read and replace, and the text that may travel. The path
//! comes from the app, so the helper trusts only what these types accept.

mod answer;
mod home;
mod location;
mod path;
mod readable;
#[cfg(test)]
mod tests;
mod text;
mod writable;

pub use answer::{FetchAnswer, FetchRefusal};
pub use home::{HOME_CONFIG_BACKUPS, HomeWrite};
pub use location::ConfigEntryLocation;
pub use path::{CONFIG_ROOTS, ConfigFilePath, InvalidConfigFilePath};
pub use text::{
    ConfigText, ConfigWrite, Fingerprint, InvalidConfigText, MAX_CONFIG_BYTES, NotWritable,
};
pub use writable::{WRITABLE_CONFIG_FILES, WRITABLE_CONFIG_FOLDERS};

/// Why a write waits for the file to stay as it was read.
pub const UNCHANGED_WHY: &str = "only when it did not change meanwhile";
