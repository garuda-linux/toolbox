#[cfg(test)]
mod tests;

use serde::Deserialize;

use crate::LoaderEntryId;

/// Only the fields the default needs. The other fields of the answer are
/// the loader's and change between bootctl versions.
#[derive(Deserialize)]
struct Listed {
    id: String,
    #[serde(rename = "isDefault", default)]
    is_default: bool,
}

/// The entry `bootctl list --json=short` marks as the default. `None` for
/// an answer that is no list or marks none, so a failed bootctl leaves the
/// default unknown.
pub fn bootctl_default(json: &str) -> Option<LoaderEntryId> {
    let listed: Vec<Listed> = serde_json::from_str(json).ok()?;

    let entry = listed.into_iter().find(|entry| entry.is_default)?;

    LoaderEntryId::new(entry.id).ok()
}
