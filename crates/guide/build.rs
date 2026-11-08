//! Builds the user guide of `docs/src` into the crate: `SUMMARY.md`,
//! every page and every picture under `images/`. Cargo reruns this when
//! anything in the folder changes, and `include_str!` makes rustc rebuild
//! the crate when a page's text changes.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

fn main() -> io::Result<()> {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let docs = manifest.join("../../docs/src").canonicalize()?;
    println!("cargo::rerun-if-changed={}", docs.display());

    let pages = files(&docs, "md")?
        .into_iter()
        .filter(|(name, _)| name != "SUMMARY.md");

    let images = files(&docs.join("images"), "png")?;

    let mut out = String::new();
    let summary = docs.join("SUMMARY.md");
    writeln!(
        out,
        "pub static SUMMARY: &str = include_str!({});",
        literal(&summary.display().to_string())
    )
    .ok();

    out.push_str("pub static PAGES: &[(&str, &str)] = &[\n");
    for (name, path) in pages {
        let path = path.display().to_string();
        writeln!(
            out,
            "    ({}, include_str!({})),",
            literal(&name),
            literal(&path)
        )
        .ok();
    }

    out.push_str("];\n");

    out.push_str("pub static IMAGES: &[(&str, &[u8])] = &[\n");
    for (name, path) in images {
        let path = path.display().to_string();
        writeln!(
            out,
            "    ({}, include_bytes!({})),",
            literal(&format!("images/{name}")),
            literal(&path)
        )
        .ok();
    }

    out.push_str("];\n");

    let target = PathBuf::from(env::var_os("OUT_DIR").unwrap_or_default()).join("embedded.rs");
    fs::write(target, out)
}

fn literal(text: &str) -> String {
    format!("\"{}\"", text.escape_default())
}

/// The files in `folder` with the `extension`, by name, sorted so the
/// build is the same each time.
fn files(folder: &Path, extension: &str) -> io::Result<Vec<(String, PathBuf)>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.extension().is_some_and(|own| own == extension)
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            found.push((name.to_owned(), path));
        }
    }

    found.sort();
    Ok(found)
}
