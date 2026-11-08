//! Finds the file of a font the desktop names, through `fontdb`.

#[cfg(test)]
mod tests;

use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::fonts::{self, FontFile};

/// How far, in CSS weight, a static face may be from the one asked for.
const NEAR_WEIGHT: u16 = 100;

/// Colour emoji fonts, the most common on Linux first.
const EMOJI_FAMILIES: [&str; 4] = [
    "Noto Color Emoji",
    "Twemoji",
    "JoyPixels",
    "Apple Color Emoji",
];

pub struct FontFinder {
    database: fontdb::Database,
}

impl FontFinder {
    /// Scans the system's font folders. Slow, call it off the UI thread.
    pub fn system() -> Self {
        let mut database = fontdb::Database::new();
        database.load_system_fonts();
        Self::from_database(database)
    }

    pub const fn from_database(database: fontdb::Database) -> Self {
        Self { database }
    }

    /// The system's colour emoji font. eframe's own fallback finds none
    /// for an emoji, and Inter and most desktop fonts draw none, so web
    /// titles would lose theirs.
    pub fn emoji(&self) -> Option<FontFile> {
        EMOJI_FAMILIES.into_iter().find_map(|family| {
            self.find(&FontSpec {
                family: family.to_owned(),
                points: 0.0,
                weight: fonts::Weight::Regular.css(),
            })
        })
    }

    /// The face of `spec`'s family closest to its weight. Desktops write
    /// family names in any case, so the name is matched without it.
    pub fn find(&self, spec: &FontSpec) -> Option<FontFile> {
        let family = self.database.faces().find_map(|face| {
            face.families
                .iter()
                .map(|(name, _)| name)
                .find(|name| name.eq_ignore_ascii_case(&spec.family))
        })?;

        let id = self.database.query(&fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: fontdb::Weight(spec.weight),
            ..fontdb::Query::default()
        })?;

        let (data, index) = self
            .database
            .with_face_data(id, |data, index| (data.to_vec(), index))?;

        // A family without the weight answers with its nearest face. A
        // static face of another weight would draw bold text regular, so
        // the bundled font draws that weight instead.
        let face_weight = self.database.face(id)?.weight.0;
        if face_weight.abs_diff(spec.weight) > NEAR_WEIGHT && !fonts::has_weight_axis(&data, index)
        {
            return None;
        }

        Some(FontFile {
            name: family.clone(),
            data,
            index,
            weight: spec.weight,
        })
    }
}
