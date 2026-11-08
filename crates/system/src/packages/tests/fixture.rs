use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// The local database captured in the Lima VM: `ALPM_DB_VERSION` and the
/// entries of gimp, gegl, mpv and yt-dlp.
const CAPTURED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/packages/fixtures/alpm/var/lib/pacman/local"
);

/// The local database layout of the captured database.
const LOCAL_DB_VERSION: &str =
    include_str!("../fixtures/alpm/var/lib/pacman/local/ALPM_DB_VERSION");

const VERSION: &str = "1.0-1";

pub const GIMP: &str = "gimp-3.2.6-2";
/// Installed as a dependency of gimp.
pub const GEGL: &str = "gegl-0.4.72-3";
/// Optionally depends on yt-dlp.
pub const MPV: &str = "mpv-1:0.41.0-7";
/// Installed explicitly in the Lima VM.
pub const YT_DLP: &str = "yt-dlp-2026.10.26-1";

pub struct System {
    folder: TempDir,
}

impl System {
    pub fn new(repos: &[&str]) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path();
        let sections = repos
            .iter()
            .fold(String::new(), |text, repo| text + "[" + repo + "]\n");

        fs::write(
            root.join("pacman.conf"),
            format!(
                "[options]\nRootDir = {}\nDBPath = {}/\n\n{sections}",
                root.display(),
                root.join("db").display()
            ),
        )
        .unwrap();

        Self { folder }
    }

    pub fn config(&self) -> PathBuf {
        self.folder.path().join("pacman.conf")
    }

    fn db(&self) -> PathBuf {
        self.folder.path().join("db")
    }

    fn local(&self) -> PathBuf {
        let local = self.db().join("local");

        fs::create_dir_all(&local).unwrap();
        fs::write(local.join("ALPM_DB_VERSION"), LOCAL_DB_VERSION).unwrap();
        local
    }

    pub fn install(&self, packages: &[&str]) {
        let versions: Vec<(&str, &str)> = packages.iter().map(|name| (*name, VERSION)).collect();

        self.install_versions(&versions);
    }

    pub fn install_versions(&self, packages: &[(&str, &str)]) {
        let local = self.local();

        for (package, version) in packages {
            let entry = local.join(format!("{package}-{version}"));

            fs::create_dir_all(&entry).unwrap();
            fs::write(entry.join("desc"), desc(package, version)).unwrap();
        }
    }

    pub fn sync(&self, repo: &str, packages: &[&str]) {
        let versions: Vec<(&str, &str)> = packages.iter().map(|name| (*name, VERSION)).collect();

        self.sync_versions(repo, &versions);
    }

    pub fn sync_versions(&self, repo: &str, packages: &[(&str, &str)]) {
        let full: Vec<(&str, &str, &str)> = packages
            .iter()
            .map(|(package, version)| (*package, *version, ""))
            .collect();

        self.sync_full(repo, &full);
    }

    pub fn path(&self) -> &Path {
        self.folder.path()
    }

    /// Copies captured local database entries, named like their folder
    /// (`mpv-1:0.41.0-7`), with every file in them.
    pub fn install_captured(&self, entries: &[&str]) {
        let local = self.local();

        for entry in entries {
            let from = Path::new(CAPTURED).join(entry);

            fs::create_dir_all(local.join(entry)).unwrap();

            for file in fs::read_dir(&from).unwrap() {
                let file = file.unwrap();

                fs::copy(file.path(), local.join(entry).join(file.file_name())).unwrap();
            }
        }
    }

    /// Appends `%KEY%` blocks to the copied `desc` of an installed entry.
    pub fn append_desc(&self, entry: &str, extra: &str) {
        let path = self.db().join("local").join(entry).join("desc");
        let desc = fs::read_to_string(&path).unwrap();

        fs::write(path, desc + extra).unwrap();
    }

    pub fn sync_full(&self, repo: &str, packages: &[(&str, &str, &str)]) {
        let sync = self.db().join("sync");
        let files: Vec<(String, String)> = packages
            .iter()
            .map(|(package, version, extra)| {
                (
                    format!("{package}-{version}/desc"),
                    desc(package, version) + extra,
                )
            })
            .collect();

        fs::create_dir_all(&sync).unwrap();
        fs::write(sync.join(format!("{repo}.db")), tar(&files)).unwrap();
    }
}

fn desc(package: &str, version: &str) -> String {
    format!(
        "%NAME%\n{package}\n\n%VERSION%\n{version}\n\n%DESC%\nAbout {package}\n\n%ARCH%\nany\n\n"
    )
}

/// An uncompressed ustar archive, which libarchive reads like the
/// compressed ones pacman downloads.
fn tar(files: &[(String, String)]) -> Vec<u8> {
    let mut archive = Vec::new();

    for (name, body) in files {
        let mut header = [0_u8; 512];

        put(&mut header[0..100], name.as_bytes());
        put(&mut header[100..108], b"0000644\0");
        put(&mut header[108..116], b"0000000\0");
        put(&mut header[116..124], b"0000000\0");
        put(
            &mut header[124..136],
            format!("{:011o}\0", body.len()).as_bytes(),
        );
        put(&mut header[136..148], b"00000000000\0");
        header[148..156].fill(b' ');
        header[156] = b'0';
        put(&mut header[257..263], b"ustar\0");
        put(&mut header[263..265], b"00");

        let sum: u32 = header.iter().map(|byte| u32::from(*byte)).sum();

        put(&mut header[148..156], format!("{sum:06o}\0 ").as_bytes());
        archive.extend_from_slice(&header);
        archive.extend_from_slice(body.as_bytes());
        archive.resize(archive.len().next_multiple_of(512), 0);
    }

    archive.resize(archive.len() + 1024, 0);
    archive
}

fn put(field: &mut [u8], value: &[u8]) {
    field[..value.len()].copy_from_slice(value);
}
