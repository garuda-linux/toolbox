use std::collections::BTreeMap;

pub use toolbox_plan::ConfigEntryLocation;
use toolbox_plan::{ConfigFilePath, FetchRefusal};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    /// `link`: a root-owned file reached through a symlink. The edit is
    /// of the file the link leads to.
    File { link: Option<Link> },
    /// The names of the files in a folder, sorted.
    Folder(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The real path, or as far as the link could be followed.
    pub target: String,
    /// Why the helper would not hand the target out. `None` for a link
    /// that is edited like any file.
    pub refusal: Option<FetchRefusal>,
}

/// What exists of the catalog, by entry id. An entry without a line is
/// missing and not shown.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing(BTreeMap<String, Found>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigEntrySpec {
    pub id: String,
    pub location: ConfigEntryLocation,
    pub directory: bool,
}

impl Listing {
    pub fn insert(&mut self, id: impl Into<String>, found: Found) {
        self.0.insert(id.into(), found);
    }

    pub fn get(&self, id: &str) -> Option<&Found> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Link {
    pub fn to(target: &ConfigFilePath) -> Self {
        Self {
            target: target.to_string(),
            refusal: None,
        }
    }

    pub fn refused(target: impl Into<String>, reason: FetchRefusal) -> Self {
        Self {
            target: target.into(),
            refusal: Some(reason),
        }
    }
}
