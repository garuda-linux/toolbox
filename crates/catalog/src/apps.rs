//! The Discover catalogs: one toggle catalog per application category in
//! `apps/`, each with one section named after its category. They come from
//! the Arch wiki's application lists (`cargo xtask wiki-sync`).

/// The category catalogs in the order the Discover page lists them.
pub const APPS: [&str; 8] = [
    "apps/documents.toml",
    "apps/games.toml",
    "apps/internet.toml",
    "apps/multimedia.toml",
    "apps/other.toml",
    "apps/science.toml",
    "apps/security.toml",
    "apps/utilities.toml",
];
