//! The colour scheme catalog, `colour-schemes.toml`: the app's own
//! palettes, which the Settings page offers in place of the desktop's.
//!
//! ```toml
//! [[scheme]]
//! id = "catppuccin-mocha"
//! name = "Catppuccin Mocha"
//! mode = "dark"
//! source = "https://github.com/catppuccin/palette"
//! licence = "MIT"
//! window = "#1e1e2e"
//! # view, text, accent, on_accent, error, warning, success, link, visited
//! tiles = { blue = "#89b4fa", green = "#a6e3a1", orange = "#fab387" }
//! ansi = ["#45475a", "…"]   # optional, all 16
//! ```

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;
use toolbox_plan::{Rgb, SchemeId};

use crate::{CatalogError, Problem, is_web_url, source};

pub const COLOUR_SCHEMES: &str = "colour-schemes.toml";

/// The terminal's normal and bright black to white.
const ANSI_COLOURS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemeMode {
    Dark,
    Light,
}

/// The validated schemes of `colour-schemes.toml`, in file order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColourSchemeCatalog {
    schemes: Vec<ColourScheme>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColourScheme {
    id: SchemeId,
    name: String,
    mode: SchemeMode,
    source: String,
    licence: String,
    colours: SchemeColours,
    tiles: SchemeTiles,
    ansi: Option<[Rgb; ANSI_COLOURS]>,
}

/// The palette's roles, as the desktop readers fill them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemeColours {
    pub window: Rgb,
    pub view: Rgb,
    pub text: Rgb,
    pub accent: Rgb,
    pub on_accent: Rgb,
    pub error: Rgb,
    pub warning: Rgb,
    pub success: Rgb,
    pub link: Rgb,
    pub visited: Rgb,
}

/// The hues of the icon tiles of the Install, System and Care pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemeTiles {
    pub blue: Rgb,
    pub green: Rgb,
    pub orange: Rgb,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    #[serde(default)]
    scheme: Vec<RawScheme>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScheme {
    id: String,
    name: String,
    mode: SchemeMode,
    source: String,
    licence: String,
    window: String,
    view: String,
    text: String,
    accent: String,
    on_accent: String,
    error: String,
    warning: String,
    success: String,
    link: String,
    visited: String,
    tiles: RawTiles,
    ansi: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTiles {
    blue: String,
    green: String,
    orange: String,
}

impl ColourSchemeCatalog {
    /// Reads and validates `colour-schemes.toml` in the catalog folder
    /// `dir`.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Read`] when the file cannot be read, otherwise the
    /// errors of [`ColourSchemeCatalog::parse`].
    pub fn load(dir: &Path) -> Result<Self, CatalogError> {
        let (text, source_name) = source::read(dir, COLOUR_SCHEMES)?;

        Self::parse(&text, &source_name)
    }

    /// Parses and validates the text of a colour scheme catalog.
    /// `source_name` names the file in errors.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Syntax`] for invalid TOML, unknown or missing fields
    /// and unknown modes, [`CatalogError::Invalid`] for the first entry
    /// that breaks a rule.
    pub fn parse(text: &str, source_name: &str) -> Result<Self, CatalogError> {
        let raw: RawCatalog = source::parse(text, source_name)?;
        let mut seen = BTreeSet::new();
        let mut schemes = Vec::with_capacity(raw.scheme.len());

        for entry in raw.scheme {
            let id = entry.id.clone();
            let scheme = checked(entry, &mut seen).map_err(|problem| CatalogError::Invalid {
                source_name: source_name.to_owned(),
                id,
                problem,
            })?;

            schemes.push(scheme);
        }

        Ok(Self { schemes })
    }

    pub fn schemes(&self) -> &[ColourScheme] {
        &self.schemes
    }

    pub fn of(&self, mode: SchemeMode) -> impl Iterator<Item = &ColourScheme> {
        self.schemes
            .iter()
            .filter(move |scheme| scheme.mode == mode)
    }

    /// The scheme `id` of `mode`. A scheme of the other mode is none, so a
    /// dark scheme never ends up as the light palette.
    pub fn find(&self, id: &SchemeId, mode: SchemeMode) -> Option<&ColourScheme> {
        self.of(mode).find(|scheme| scheme.id == *id)
    }
}

impl ColourScheme {
    pub const fn id(&self) -> &SchemeId {
        &self.id
    }

    /// The English name, translated in the UI.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn mode(&self) -> SchemeMode {
        self.mode
    }

    /// Where the colours come from, an `https` link.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The SPDX licence of the source.
    pub fn licence(&self) -> &str {
        &self.licence
    }

    pub const fn colours(&self) -> &SchemeColours {
        &self.colours
    }

    pub const fn tiles(&self) -> SchemeTiles {
        self.tiles
    }

    /// The scheme's own 16 terminal colours, `None` where it names none.
    pub const fn ansi(&self) -> Option<&[Rgb; ANSI_COLOURS]> {
        self.ansi.as_ref()
    }
}

fn checked(entry: RawScheme, seen: &mut BTreeSet<String>) -> Result<ColourScheme, Problem> {
    source::claim_id(&entry.id, seen)?;
    let id = SchemeId::new(&entry.id).map_err(|_invalid| Problem::BadId)?;

    if source::is_blank(&entry.name) {
        return Err(Problem::EmptyName);
    }

    if !is_web_url(&entry.source) {
        return Err(Problem::BadUrl(entry.source));
    }

    if source::is_blank(&entry.licence) {
        return Err(Problem::EmptyLicence);
    }

    let colours = SchemeColours {
        window: colour(&entry.window)?,
        view: colour(&entry.view)?,
        text: colour(&entry.text)?,
        accent: colour(&entry.accent)?,
        on_accent: colour(&entry.on_accent)?,
        error: colour(&entry.error)?,
        warning: colour(&entry.warning)?,
        success: colour(&entry.success)?,
        link: colour(&entry.link)?,
        visited: colour(&entry.visited)?,
    };

    let tiles = SchemeTiles {
        blue: colour(&entry.tiles.blue)?,
        green: colour(&entry.tiles.green)?,
        orange: colour(&entry.tiles.orange)?,
    };

    let ansi = entry.ansi.as_deref().map(ansi).transpose()?;

    Ok(ColourScheme {
        id,
        name: entry.name,
        mode: entry.mode,
        source: entry.source,
        licence: entry.licence,
        colours,
        tiles,
        ansi,
    })
}

fn colour(text: &str) -> Result<Rgb, Problem> {
    Rgb::parse(text).map_err(Problem::BadColour)
}

fn ansi(texts: &[String]) -> Result<[Rgb; ANSI_COLOURS], Problem> {
    let colours = texts
        .iter()
        .map(|text| colour(text))
        .collect::<Result<Vec<_>, _>>()?;

    <[Rgb; ANSI_COLOURS]>::try_from(colours).map_err(|_count| Problem::AnsiCount(texts.len()))
}
