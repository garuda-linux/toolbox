//! What every `list` command takes: `--json`, `--search` and the filters
//! of its own rows.

#[cfg(test)]
mod tests;

use clap::Args;

use super::Read;

/// The output of a `list`: JSON or text, of the rows a search keeps.
#[derive(Clone, Debug, Default, Args)]
pub struct ReadList {
    /// Print JSON instead of text, for scripts.
    #[arg(long)]
    pub json: bool,

    /// Only the rows whose name contains this text, in any case.
    #[arg(long, value_name = "TEXT")]
    pub search: Option<Search>,
}

impl ReadList {
    pub const fn read(&self) -> Read {
        Read { json: self.json }
    }
}

/// A text to find in a row's names, whatever its case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Search(String);

impl Search {
    pub fn new(text: &str) -> Self {
        Self(text.to_lowercase())
    }

    pub fn matches<'a>(&self, names: impl IntoIterator<Item = &'a str>) -> bool {
        names
            .into_iter()
            .any(|name| name.to_lowercase().contains(&self.0))
    }
}

#[derive(Debug, Args)]
pub struct KernelsList {
    /// Only the installed kernels.
    #[arg(long)]
    pub installed: bool,

    #[command(flatten)]
    pub read: ReadList,
}

#[derive(Debug, Args)]
pub struct LocalesList {
    /// Only the enabled locales.
    #[arg(long)]
    pub enabled: bool,

    #[command(flatten)]
    pub read: ReadList,
}

impl std::str::FromStr for Search {
    type Err = std::convert::Infallible;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(text))
    }
}
