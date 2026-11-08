use toolbox_catalog::{ConfigFileCatalog, ConfigSection};
use toolbox_core::features::configs::ConfigEntrySpec;

pub fn specs(catalog: &ConfigFileCatalog) -> Vec<ConfigEntrySpec> {
    catalog
        .sections()
        .iter()
        .flat_map(ConfigSection::entries)
        .map(|entry| ConfigEntrySpec {
            id: entry.id().to_owned(),
            location: entry.location().clone(),
            directory: entry.is_directory(),
        })
        .collect()
}
