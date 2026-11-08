//! The Packages page's reads: the index and one package's details.

use toolbox_plan::PackageName;
use toolbox_state::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, OptionalDependency, Origin,
};

use super::fixture::System;
use crate::ProbeError;
use crate::packages::{Libalpm, PackageIndexSource as _};

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn system() -> System {
    let system = System::new(&["core", "extra"]);

    system.install_full(
        "gimp",
        "%SIZE%\n4096\n\n%REASON%\n0\n\n%INSTALLDATE%\n1700000000\n\n\
         %URL%\nhttps://www.gimp.org/\n\n%LICENSE%\nGPL-3.0-or-later\n\n\
         %GROUPS%\ngraphics\n\n%DEPENDS%\ngegl>=0.4\nsh\n\n\
         %OPTDEPENDS%\nghostscript: for postscript support\n\n\
         %PROVIDES%\ngimp-editor\n\n%CONFLICTS%\ngimp-git\n\n%REPLACES%\ngimp-old\n\n",
        &["usr/", "usr/bin/", "usr/bin/gimp"],
    );
    system.install_full("gegl", "%SIZE%\n2048\n\n%REASON%\n1\n\n", &[]);
    system.install_full("paru", "%SIZE%\n10\n\n", &[]);
    system.sync_full(
        "core",
        &[
            ("gegl", "1.0-1", "%ISIZE%\n2000\n\n"),
            ("gimp", "2.0-1", ""),
        ],
    );
    system.sync_full(
        "extra",
        &[
            ("gimp", "3.0-1", ""),
            ("inkscape", "1.4-1", "%ISIZE%\n9000\n\n"),
        ],
    );
    system
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

    assert_eq!(names, ["gegl", "gimp", "inkscape", "paru"]);

    assert_eq!(
        index.entries[1],
        IndexEntry {
            name: name("gimp"),
            description: "About gimp".to_owned(),
            version: "2.0-1".to_owned(),
            origin: Origin::Repository("core".to_owned()),
            size: 4096,
            installed: Some(Installed {
                version: "1.0-1".to_owned(),
                reason: InstallReason::Explicit,
            }),
        },
        "the first repository wins, the size is the installed build's"
    );
    assert_eq!(index.entries[2].size, 9000);
    assert_eq!(index.entries[2].installed, None);
    assert_eq!(index.entries[3].origin, Origin::Foreign);
}

#[test]
fn the_index_carries_the_installed_dependency_graph() {
    let system = system();
    let index = Libalpm::with_config(system.config()).index().unwrap();

    assert!(index.local.contains(&LocalNode {
        name: name("gimp"),
        depends: vec!["gegl>=0.4".to_owned(), "sh".to_owned()],
        provides: vec!["gimp-editor".to_owned()],
        reason: InstallReason::Explicit,
    }));
    assert!(
        index
            .local
            .iter()
            .any(|node| node.name.as_str() == "gegl" && node.reason == InstallReason::Dependency)
    );
}

#[test]
fn without_a_synced_database_the_index_is_unavailable() {
    let system = System::new(&["core"]);
    system.install(&["bash"]);

    assert_eq!(
        Libalpm::with_config(system.config()).index(),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn the_details_of_an_installed_package_come_from_the_local_database() {
    let system = system();
    let details = Libalpm::with_config(system.config())
        .details(&name("gimp"))
        .unwrap()
        .unwrap();

    assert_eq!(details.origin, Origin::Repository("core".to_owned()));
    assert_eq!(details.url.as_deref(), Some("https://www.gimp.org/"));
    assert_eq!(details.licenses, ["GPL-3.0-or-later"]);
    assert_eq!(details.groups, ["graphics"]);
    assert_eq!(details.depends, ["gegl>=0.4", "sh"]);
    assert_eq!(
        details.optional,
        [OptionalDependency {
            name: "ghostscript".to_owned(),
            reason: "for postscript support".to_owned(),
        }]
    );
    assert_eq!(details.provides, ["gimp-editor"]);
    assert_eq!(details.conflicts, ["gimp-git"]);
    assert_eq!(details.replaces, ["gimp-old"]);
    assert_eq!(details.size, 4096);

    let installed = details.installed.unwrap();
    assert_eq!(installed.reason, InstallReason::Explicit);
    assert_eq!(installed.date, Some(1_700_000_000));
    assert_eq!(installed.files, ["/usr/", "/usr/bin/", "/usr/bin/gimp"]);
}

#[test]
fn a_dependency_names_who_requires_it() {
    let system = system();
    let details = Libalpm::with_config(system.config())
        .details(&name("gegl"))
        .unwrap()
        .unwrap();

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
