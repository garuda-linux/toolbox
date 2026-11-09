#[cfg(test)]
mod tests;

use std::fs::{self, File};
use std::io::{self, Read as _, Seek as _, SeekFrom};
use std::path::{Path, PathBuf};

use toolbox_plan::sdboot::{
    ENTRIES_FOLDER, HEADER_BYTES, LOADER_CONF, LoaderEntry, LoaderSettings, MAX_ENTRIES,
    SdBootMenu, UKI_FOLDER, esp_place, sections,
};

const MOUNTINFO: &str = "proc/self/mountinfo";

/// An automount shows the ESP only once something opens it.
const ESP_CANDIDATES: [&str; 3] = ["efi", "boot", "boot/efi"];

const MAX_TEXT_BYTES: u64 = 64 << 10;

/// Containers and snap mounts make tables far past 64 KiB, and a cut one
/// can hide the ESP line.
const MAX_MOUNTINFO_BYTES: u64 = 1 << 20;

const MAX_OS_RELEASE_BYTES: u32 = 64 << 10;

/// `.uname` holds one kernel release.
const MAX_UNAME_BYTES: u32 = 256;

struct Listing {
    found: Vec<(String, PathBuf)>,
    skipped: usize,
    cut: usize,
}

/// # Errors
///
/// When no ESP is mounted, or the mount table or `loader.conf` cannot be
/// read. A file that is no entry is left out instead.
pub fn sdboot_menu(root: &Path) -> io::Result<SdBootMenu> {
    let esp = root.join(esp(root)?);
    let settings = match text(&esp.join(LOADER_CONF)) {
        Ok(conf) => LoaderSettings::parse(&conf),
        Err(error) if error.kind() == io::ErrorKind::NotFound => LoaderSettings::default(),
        Err(error) => return Err(error),
    };

    let ukis = files(&esp.join(UKI_FOLDER), ".efi")?;
    let type_one = files(&esp.join(ENTRIES_FOLDER), ".conf")?;
    let listed = ukis.found.len() + type_one.found.len();

    let entries: Vec<LoaderEntry> = ukis
        .found
        .iter()
        .filter_map(|(name, path)| uki(name, path))
        .chain(
            type_one
                .found
                .iter()
                .filter_map(|(name, path)| LoaderEntry::type_one(name, &text(path).ok()?)),
        )
        .collect();

    let mut menu = SdBootMenu {
        settings,
        skipped: ukis.skipped + type_one.skipped + (listed - entries.len()),
        cut: ukis.cut + type_one.cut,
        entries,
    };

    menu.cut_to_limit();
    Ok(menu)
}

pub fn esp(root: &Path) -> io::Result<&'static str> {
    let place = || {
        text_up_to(&root.join(MOUNTINFO), MAX_MOUNTINFO_BYTES)
            .map(|mountinfo| esp_place(&mountinfo))
    };

    if let Some(place) = place()? {
        return Ok(place);
    }

    for candidate in ESP_CANDIDATES {
        // The lookup alone mounts it, whatever it answers.
        drop(fs::metadata(root.join(candidate).join("EFI")));
    }

    place()?.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no ESP is mounted"))
}

/// The ESP is FAT, which has no links, so a plain listing is all there is.
fn files(folder: &Path, suffix: &str) -> io::Result<Listing> {
    let mut listing = Listing {
        found: Vec::new(),
        skipped: 0,
        cut: 0,
    };

    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(listing),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let Ok(entry) = entry else {
            listing.skipped += 1;
            continue;
        };

        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }

        let Ok(name) = entry.file_name().into_string() else {
            listing.skipped += 1;
            continue;
        };

        if !name.to_ascii_lowercase().ends_with(suffix) {
            continue;
        }

        if listing.found.len() == MAX_ENTRIES {
            listing.cut += 1;
        } else {
            listing.found.push((name, entry.path()));
        }
    }

    listing.found.sort();
    Ok(listing)
}

/// A UKI the header reader cannot make out still boots, so it keeps its
/// place with its name as title.
fn uki(name: &str, path: &Path) -> Option<LoaderEntry> {
    let mut file = File::open(path).ok()?;
    let mut header = Vec::new();

    (&mut file)
        .take(u64::try_from(HEADER_BYTES).unwrap_or(u64::MAX))
        .read_to_end(&mut header)
        .ok()?;

    let found = sections(&header);
    let mut section = |wanted: &str, max: u32| {
        let section = found.iter().find(|section| section.name == wanted)?;

        file.seek(SeekFrom::Start(section.offset.into())).ok()?;

        let mut content = Vec::new();

        (&mut file)
            .take(section.size.min(max).into())
            .read_to_end(&mut content)
            .ok()?;

        Some(String::from_utf8_lossy(&content).into_owned())
    };

    let os_release = section(".osrel", MAX_OS_RELEASE_BYTES);

    let uname = section(".uname", MAX_UNAME_BYTES);

    LoaderEntry::uki(name, os_release.as_deref().unwrap_or(""), uname.as_deref())
}

fn text(path: &Path) -> io::Result<String> {
    text_up_to(path, MAX_TEXT_BYTES)
}

fn text_up_to(path: &Path, max: u64) -> io::Result<String> {
    let mut content = Vec::new();

    File::open(path)?.take(max).read_to_end(&mut content)?;

    String::from_utf8(content)
        .map_err(|_not_text| io::Error::new(io::ErrorKind::InvalidData, "the file is no text"))
}
