//! The Packages page's reads: gimp and gegl from the local database
//! captured in the Lima VM, pacman's log of the same install, and the
//! rows around them that no capture holds.

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, Origin, PackageDetails, PackageIndex,
};
use toolbox_plan::PackageName;
use toolbox_system::{Libalpm, PackageIndexSource, ProbeError, RecentLogSource};

const CAPTURED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/system/src/packages/fixtures/alpm/var/lib/pacman/local"
);

/// The entries copied from the capture, named like their folders.
const ENTRIES: [&str; 3] = ["ALPM_DB_VERSION", "gegl-0.4.72-3", "gimp-3.2.6-2"];

/// pacman's log of the install that put gimp 3.2.6-2 on the Lima VM.
const PACMAN_LOG: &str =
    include_str!("../../../../../crates/system/src/status/pacman_log/fixtures/pacman.log");

/// gimp and gegl as captured, babl, a leftover dependency, paru built
/// from the AUR, and inkscape and krita on offer.
pub struct Database {
    alpm: Libalpm,
    _folder: TempDir,
}

/// pacman's log as captured.
pub struct Log;

impl Database {
    /// A copy of the capture, so libalpm never writes beside it.
    pub fn captured() -> Self {
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path();
        let db = root.join("db");
        let local = db.join("local");

        for entry in ENTRIES {
            copy(&Path::new(CAPTURED).join(entry), &local.join(entry));
        }

        let config = root.join("pacman.conf");
        fs::write(
            &config,
            format!(
                "[options]\nRootDir = {}\nDBPath = {}/\n",
                root.display(),
                db.display()
            ),
        )
        .unwrap();

        Self {
            alpm: Libalpm::with_config(config),
            _folder: folder,
        }
    }

    /// One captured package. Edited: its origin is `extra`, where Arch
    /// ships gimp and gegl, as no sync database was captured and libalpm
    /// would call it foreign.
    pub fn package(&self, package: &str) -> PackageDetails {
        let mut details = self.alpm.details(&name(package)).unwrap().unwrap();
        details.origin = Origin::Repository("extra".to_owned());
        details
    }
}

impl PackageIndexSource for Database {
    fn index(&self) -> Result<PackageIndex, ProbeError> {
        let gegl = self.package("gegl");
        let gimp = self.package("gimp");
        let dependency = Some(InstallReason::Dependency);
        let explicit = Some(InstallReason::Explicit);

        Ok(PackageIndex {
            entries: vec![
                entry(
                    "babl",
                    "Dynamic, any to any, pixel format conversion library",
                    dependency,
                ),
                listed(&gegl),
                listed(&gimp),
                entry("inkscape", "Professional vector graphics editor", None),
                entry("krita", "Edit and paint images", None),
                entry("leftover", "A library nothing needs anymore", dependency),
                entry("paru", "Feature packed AUR helper", explicit),
            ],
            local: vec![
                node("babl", InstallReason::Dependency),
                local(&gegl),
                local(&gimp),
                node("leftover", InstallReason::Dependency),
                node("paru", InstallReason::Explicit),
            ],
        })
    }

    fn details(&self, package: &PackageName) -> Result<Option<PackageDetails>, ProbeError> {
        match package.as_str() {
            "gimp" | "gegl" => Ok(Some(self.package(package.as_str()))),
            _ => Ok(None),
        }
    }
}

impl RecentLogSource for Log {
    fn recent(&self) -> Result<String, ProbeError> {
        Ok(PACMAN_LOG.to_owned())
    }
}

fn copy(from: &Path, to: &Path) {
    if from.is_file() {
        fs::create_dir_all(to.parent().unwrap()).unwrap();
        fs::copy(from, to).unwrap();
        return;
    }

    fs::create_dir_all(to).unwrap();
    for file in fs::read_dir(from).unwrap() {
        let file = file.unwrap();
        fs::copy(file.path(), to.join(file.file_name())).unwrap();
    }
}

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

/// A captured package as the index lists it.
fn listed(details: &PackageDetails) -> IndexEntry {
    IndexEntry {
        name: details.name.clone(),
        description: details.description.clone(),
        version: details.version.clone(),
        origin: details.origin.clone(),
        size: details.size,
        installed: details.installed.as_ref().map(|installed| Installed {
            version: installed.version.clone(),
            reason: installed.reason,
        }),
    }
}

/// A captured package in the installed packages' graph.
fn local(details: &PackageDetails) -> LocalNode {
    LocalNode {
        name: details.name.clone(),
        depends: details.depends.clone(),
        provides: details.provides.clone(),
        reason: details
            .installed
            .as_ref()
            .map_or(InstallReason::Explicit, |installed| installed.reason),
    }
}

fn entry(text: &str, description: &str, installed: Option<InstallReason>) -> IndexEntry {
    IndexEntry {
        name: name(text),
        description: description.to_owned(),
        version: "3.0.4-1".to_owned(),
        origin: if text == "paru" {
            Origin::Foreign
        } else {
            Origin::Repository("extra".to_owned())
        },
        size: 37 * 1024 * 1024,
        installed: installed.map(|reason| Installed {
            version: "3.0.4-1".to_owned(),
            reason,
        }),
    }
}

fn node(text: &str, reason: InstallReason) -> LocalNode {
    LocalNode {
        name: name(text),
        depends: Vec::new(),
        provides: Vec::new(),
        reason,
    }
}
