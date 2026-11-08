use crate::{ConfigFilePath, HomePath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigEntryLocation {
    /// Below the user's home: opened with the desktop's default app.
    Home(HomePath),
    /// Root-owned: edited as a copy and written back by the helper.
    Root(ConfigFilePath),
}
