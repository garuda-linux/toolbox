//! The user guide of `docs/src`, the same pages mdBook builds, inside the
//! app. `SUMMARY.md` gives the order and the parts, each page turns into
//! blocks the window draws with its own widgets and the command line
//! prints.
//!
//! `build.rs` builds every page and picture into the binary, so the guide
//! matches the app's version. Editing a page rebuilds
//! this crate.

#[cfg(test)]
mod tests;

mod guide;
pub mod markdown;
mod search;
pub mod summary;

mod built_in {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

use std::sync::LazyLock;

pub use guide::{Guide, GuidePart, Outline, Page};
pub use markdown::{
    Block, Callout, CalloutKind, Cell, Code, Heading, Image, Link, List, Span, Style, Table,
};
pub use search::{Hit, MAX_HITS};
pub use summary::{Entry, Part};

static GUIDE: LazyLock<Guide> = LazyLock::new(|| Guide::new(built_in::SUMMARY, built_in::PAGES));

pub const FIRST_PAGE: &str = "introduction.md";

/// The guide built into the binary, parsed on first use.
pub fn embedded() -> &'static Guide {
    &GUIDE
}

/// The bytes of a picture the pages show, by its path under `docs/src`.
pub fn image(path: &str) -> Option<&'static [u8]> {
    let path = path.strip_prefix("./").unwrap_or(path);
    built_in::IMAGES
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, bytes)| *bytes)
}

/// Width and height of a PNG from its header, so the page can keep the
/// picture's room before it is decoded. `None` for anything else.
pub fn image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

    // The signature, the first chunk's length, then `IHDR`.
    const WIDTH_AT: usize = 16;

    if !bytes.starts_with(SIGNATURE) || bytes.get(12..16) != Some(b"IHDR".as_slice()) {
        return None;
    }

    let number = |at: usize| {
        bytes
            .get(at..at + 4)
            .and_then(|word| word.try_into().ok())
            .map(u32::from_be_bytes)
    };

    Some((number(WIDTH_AT)?, number(WIDTH_AT + 4)?))
}
