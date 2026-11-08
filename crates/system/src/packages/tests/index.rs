//! The Packages page's reads: the index and one package's details. The
//! local database is the captured one. No sync database was captured, so
//! the repositories are built by the fixture.

use toolbox_plan::PackageName;
use toolbox_state::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, OptionalDependency, Origin, PackageDetails,
};

use super::fixture::{GEGL, GIMP, MPV, System, YT_DLP};
use crate::ProbeError;
use crate::packages::{Libalpm, PackageIndexSource as _};

const GIMP_DESC: &str = include_str!("../fixtures/alpm/var/lib/pacman/local/gimp-3.2.6-2/desc");
const GIMP_FILES: &str = include_str!("../fixtures/alpm/var/lib/pacman/local/gimp-3.2.6-2/files");
const GEGL_DESC: &str = include_str!("../fixtures/alpm/var/lib/pacman/local/gegl-0.4.72-3/desc");

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

/// The lines of one `%KEY%` block of a captured `desc` or `files`.
fn block(text: &str, key: &str) -> Vec<String> {
    text.split("\n\n")
        .find_map(|block| block.strip_prefix(&format!("%{key}%\n")))
        .map(|lines| lines.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

fn system() -> System {
    let system = System::new(&["core", "extra"]);

    system.install_captured(&[GIMP, GEGL, MPV, YT_DLP]);
    system.sync_full(
        "core",
        &[
            ("gegl", "0.4.72-3", "%ISIZE%\n2000\n\n"),
            ("gimp", "3.2.6-2", ""),
        ],
    );
    system.sync_full(
        "extra",
        &[
            ("gimp", "3.2.8-1", ""),
            ("inkscape", "1.4-1", "%ISIZE%\n9000\n\n"),
        ],
    );
    system
}

fn details(system: &System, package: &str) -> PackageDetails {
    Libalpm::with_config(system.config())
        .details(&name(package))
        .unwrap()
        .unwrap()
}

#[test]
fn the_index_holds_every_offered_and_every_installed_package_by_name() {
    let system = system();
    let index = Libalpm::with_config(system.config()).index().unwrap();

    let names: Vec<&str> = index
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();

    assert_eq!(names, ["gegl", "gimp", "inkscape", "mpv", "yt-dlp"]);

    assert_eq!(
        index.entries[1],
        IndexEntry {
            name: name("gimp"),
            description: "About gimp".to_owned(),
            version: "3.2.6-2".to_owned(),
            origin: Origin::Repository("core".to_owned()),
            size: 158_547_676,
            installed: Some(Installed {
                version: "3.2.6-2".to_owned(),
                reason: InstallReason::Explicit,
            }),
        },
        "the first repository wins, the size is the installed build's"
    );
    assert_eq!(index.entries[2].size, 9000);
    assert_eq!(index.entries[2].installed, None);
    assert_eq!(
        index.entries[3],
        IndexEntry {
            name: name("mpv"),
            description: "a free, open source, and cross-platform media player".to_owned(),
            version: "1:0.41.0-7".to_owned(),
            origin: Origin::Foreign,
            size: 6_656_527,
            installed: Some(Installed {
                version: "1:0.41.0-7".to_owned(),
                reason: InstallReason::Explicit,
            }),
        }
    );
}

#[test]
fn the_index_carries_the_installed_dependency_graph() {
    let system = system();
    let index = Libalpm::with_config(system.config()).index().unwrap();
    let gimp = index.local.iter().find(|node| node.name.as_str() == "gimp");

    assert_eq!(
        gimp,
        Some(&LocalNode {
            name: name("gimp"),
            depends: block(GIMP_DESC, "DEPENDS"),
            provides: Vec::new(),
            reason: InstallReason::Explicit,
        })
    );
    assert!(gimp.unwrap().depends.contains(&"gegl".to_owned()));
    assert!(index.local.contains(&LocalNode {
        name: name("gegl"),
        depends: block(GEGL_DESC, "DEPENDS"),
        provides: vec![
            "libgegl-0.4.so=0-64".to_owned(),
            "libgegl-npd-0.4.so=libgegl-npd-0.4.so-64".to_owned(),
            "libgegl-sc-0.4.so=libgegl-sc-0.4.so-64".to_owned(),
        ],
        reason: InstallReason::Dependency,
    }));
}

#[test]
fn without_a_synced_database_the_index_is_unavailable() {
    let system = System::new(&["core"]);
    system.install_captured(&[GIMP]);

    assert_eq!(
        Libalpm::with_config(system.config()).index(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn the_details_of_an_installed_package_come_from_the_local_database() {
    let details = details(&system(), "gimp");

    assert_eq!(details.origin, Origin::Repository("core".to_owned()));
    assert_eq!(details.description, "GNU Image Manipulation Program");
    assert_eq!(details.url.as_deref(), Some("https://www.gimp.org/"));
    assert_eq!(details.licenses, ["GPL-3.0-or-later"]);
    assert_eq!(details.groups, Vec::<String>::new());
    assert_eq!(details.depends, block(GIMP_DESC, "DEPENDS"));
    assert_eq!(details.optional.len(), 7);
    assert_eq!(
        details.optional[6],
        OptionalDependency {
            name: "gvfs".to_owned(),
            reason: "for HTTP/S support (and many other schemes)".to_owned(),
        }
    );
    assert_eq!(details.provides, Vec::<String>::new());
    assert_eq!(details.size, 158_547_676);

    let installed = details.installed.unwrap();
    let files: Vec<String> = block(GIMP_FILES, "FILES")
        .iter()
        .map(|file| format!("/{file}"))
        .collect();

    assert_eq!(installed.reason, InstallReason::Explicit);
    assert_eq!(installed.date, Some(1_794_155_577));
    assert_eq!(files.len(), 6537);
    assert_eq!(
        &installed.files[..3],
        ["/etc/", "/etc/gimp/", "/etc/gimp/3.0/"]
    );
    assert_eq!(installed.files, files);
}

/// The captured gimp has no groups, conflicts or replaces. These blocks
/// are appended to its `desc`.
#[test]
fn groups_conflicts_and_replaces_are_read() {
    let system = system();

    system.append_desc(
        GIMP,
        "%GROUPS%\ngraphics\n\n%CONFLICTS%\ngimp-git\n\n%REPLACES%\ngimp-old\n\n",
    );

    let details = details(&system, "gimp");

    assert_eq!(details.groups, ["graphics"]);
    assert_eq!(details.conflicts, ["gimp-git"]);
    assert_eq!(details.replaces, ["gimp-old"]);
}

#[test]
fn a_dependency_names_who_requires_it() {
    let details = details(&system(), "gegl");

    assert_eq!(details.required_by, ["gimp"]);
    assert_eq!(
        details.installed.map(|installed| installed.reason),
        Some(InstallReason::Dependency)
    );
}

#[test]
fn the_details_of_an_offered_package_come_from_its_repository() {
    let system = system();
    let alpm = Libalpm::with_config(system.config());
    let details = alpm.details(&name("inkscape")).unwrap().unwrap();

    assert_eq!(details.origin, Origin::Repository("extra".to_owned()));
    assert_eq!(details.version, "1.4-1");
    assert_eq!(details.installed, None);
    assert_eq!(alpm.details(&name("nothing")), Ok(None));
}
