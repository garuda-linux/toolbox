//! App icons from the `AppStream` icon cache, for packages whose name is no
//! icon of the theme. `archlinux-appstream-data` installs one PNG per app
//! as `<package>_<icon>.png` in `<root>/<repository>/<size>/`.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::ops::Bound;
use std::path::{Path, PathBuf};

/// Where the `AppStream` data puts its icons: the current location first,
/// then the one of older releases.
const SYSTEM_ROOTS: [&str; 2] = ["/usr/share/swcatalog/icons", "/usr/share/app-info/icons"];

/// The sizes the cache holds, largest first: a list row scales down well.
const SIZES: [&str; 3] = ["128x128", "64x64", "48x48"];

/// Arch ships its icon cache as JPEG XL, older caches as PNG, some as
/// SVG.
const EXTENSIONS: [&str; 3] = ["jxl", "png", "svg"];

pub trait AppIconSource: Send + Sync {
    fn icon_file(&self, package: &str) -> Option<PathBuf>;
}

/// [`AppIconSource`] over the `AppStream` icon cache, read once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppStreamIcons {
    /// Every icon by file name, the largest size of a name kept.
    files: BTreeMap<String, PathBuf>,
}

impl AppStreamIcons {
    /// Reads the system's icon cache. Blocks on the file system.
    pub fn system() -> Self {
        let roots: Vec<&Path> = SYSTEM_ROOTS.iter().map(Path::new).collect();

        Self::scan(&roots)
    }

    /// Missing folders count as empty.
    pub fn scan(roots: &[&Path]) -> Self {
        let mut files = BTreeMap::new();

        for size in SIZES {
            for repository in roots.iter().flat_map(|root| subfolders(root)) {
                for (name, path) in icons_in(&repository.join(size)) {
                    files.entry(name).or_insert(path);
                }
            }
        }

        Self { files }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the cache holds no icon (no `AppStream` data installed).
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl AppIconSource for AppStreamIcons {
    /// `<package>.<extension>` first, else the first `<package>_<icon>`.
    fn icon_file(&self, package: &str) -> Option<PathBuf> {
        if package.is_empty() {
            return None;
        }

        let exact = EXTENSIONS
            .map(|extension| format!("{package}.{extension}"))
            .into_iter()
            .find_map(|name| self.files.get(&name).cloned());

        exact.or_else(|| {
            let prefix = format!("{package}_");

            self.files
                .range::<str, _>((Bound::Included(prefix.as_str()), Bound::Unbounded))
                .next()
                .filter(|(name, _)| name.starts_with(&prefix))
                .map(|(_, path)| path.clone())
        })
    }
}

/// `path` as a `file://` URL for the UI, with every byte outside the
/// unreserved set and `/` percent-encoded.
pub fn file_url(path: &Path) -> String {
    let mut url = String::from("file://");

    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            url.push(char::from(byte));
        } else {
            // Writing to a String cannot fail.
            write!(url, "%{byte:02X}").ok();
        }
    }

    url
}

/// The local path of a `file://` URL, as a file chooser hands it back.
/// `None` for another scheme, a host, a broken escape or a path that is
/// no UTF-8.
pub fn file_path(url: &str) -> Option<String> {
    let encoded = url.strip_prefix("file://")?;
    if !encoded.starts_with('/') {
        return None;
    }

    let mut bytes = Vec::with_capacity(encoded.len());
    let mut rest = encoded.bytes();
    while let Some(byte) = rest.next() {
        if byte == b'%' {
            let high = char::from(rest.next()?).to_digit(16)?;
            let low = char::from(rest.next()?).to_digit(16)?;
            bytes.push(u8::try_from(high * 16 + low).ok()?);
        } else {
            bytes.push(byte);
        }
    }

    String::from_utf8(bytes).ok()
}

fn subfolders(root: &Path) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();

    folders.sort();
    folders
}

fn icons_in(folder: &Path) -> Vec<(String, PathBuf)> {
    fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            let picture = Path::new(&name)
                .extension()
                .is_some_and(|extension| EXTENSIONS.iter().any(|known| extension == *known));

            picture.then(|| (name, entry.path()))
        })
        .collect()
}
