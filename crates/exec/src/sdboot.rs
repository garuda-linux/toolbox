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
const MAX_OS_RELEASE_BYTES: u32 = 64 << 10;

/// `.uname` holds one kernel release.
const MAX_UNAME_BYTES: u32 = 256;

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

    let mut entries: Vec<LoaderEntry> = files(&esp.join(UKI_FOLDER), ".efi")?
        .into_iter()
        .filter_map(|(name, path)| uki(&name, &path))
        .chain(
            files(&esp.join(ENTRIES_FOLDER), ".conf")?
                .into_iter()
                .filter_map(|(name, path)| LoaderEntry::type_one(&name, &text(&path).ok()?)),
        )
        .collect();

    entries.truncate(MAX_ENTRIES);

    Ok(SdBootMenu { settings, entries })
}

pub fn esp(root: &Path) -> io::Result<&'static str> {
    let place = || text(&root.join(MOUNTINFO)).map(|mountinfo| esp_place(&mountinfo));

    if let Some(place) = place()? {
        return Ok(place);
    }

    for candidate in ESP_CANDIDATES {
        let _opened = root.join(candidate).join("EFI").exists();
    }

    place()?.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no ESP is mounted"))
}

/// The ESP is FAT, which has no links, so a plain listing is all there is.
fn files(folder: &Path, suffix: &str) -> io::Result<Vec<(String, PathBuf)>> {
    let listing = match fs::read_dir(folder) {
        Ok(listing) => listing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let mut found: Vec<(String, PathBuf)> = listing
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| !kind.is_dir()))
        .filter_map(|entry| Some((entry.file_name().into_string().ok()?, entry.path())))
        .filter(|(name, _)| name.to_ascii_lowercase().ends_with(suffix))
        .take(MAX_ENTRIES)
        .collect();

    found.sort();
    Ok(found)
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
    let mut content = Vec::new();

    File::open(path)?
        .take(MAX_TEXT_BYTES)
        .read_to_end(&mut content)?;

    String::from_utf8(content)
        .map_err(|_not_text| io::Error::new(io::ErrorKind::InvalidData, "the file is no text"))
}
