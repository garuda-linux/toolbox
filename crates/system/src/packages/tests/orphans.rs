use toolbox_plan::PackageName;

use super::fixture::System;
use crate::ProbeError;
use crate::packages::{Libalpm, OrphanSource as _};

const AS_DEPENDENCY: &str = "%REASON%\n1\n\n";

fn names(names: &[&str]) -> Vec<PackageName> {
    names
        .iter()
        .map(|name| PackageName::new(*name).unwrap())
        .collect()
}

#[test]
fn a_dependency_nothing_requires_is_an_orphan() {
    let system = System::new(&["core"]);

    system.install_full("gimp", "%DEPENDS%\ngegl\n\n", &[]);
    system.install_full("gegl", AS_DEPENDENCY, &[]);
    system.install_full("old-lib", AS_DEPENDENCY, &[]);
    system.install_full("zlib-ng", AS_DEPENDENCY, &[]);

    assert_eq!(
        Libalpm::with_config(system.config()).orphans(),
        Ok(names(&["old-lib", "zlib-ng"]))
    );
}

#[test]
fn an_explicit_package_is_never_an_orphan() {
    let system = System::new(&["core"]);

    system.install(&["firefox"]);

    assert_eq!(
        Libalpm::with_config(system.config()).orphans(),
        Ok(Vec::new())
    );
}

#[test]
fn an_optional_dependency_in_use_is_kept() {
    let system = System::new(&["core"]);

    system.install_full("mpv", "%OPTDEPENDS%\nyt-dlp: online videos\n\n", &[]);
    system.install_full("yt-dlp", AS_DEPENDENCY, &[]);

    assert_eq!(
        Libalpm::with_config(system.config()).orphans(),
        Ok(Vec::new())
    );
}

#[test]
fn without_a_local_database_orphans_are_unavailable() {
    let system = System::new(&["core"]);

    assert_eq!(
        Libalpm::with_config(system.config()).orphans(),
        Err(ProbeError::Unavailable)
    );
}
