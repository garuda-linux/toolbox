use toolbox_plan::PackageName;

use super::fixture::{GEGL, GIMP, MPV, System, YT_DLP};
use crate::ProbeError;
use crate::packages::{Libalpm, OrphanSource as _};

fn names(names: &[&str]) -> Vec<PackageName> {
    names
        .iter()
        .map(|name| PackageName::new(*name).unwrap())
        .collect()
}

fn orphans(system: &System) -> Result<Vec<PackageName>, ProbeError> {
    Libalpm::with_config(system.config()).orphans()
}

/// The captured database without gimp leaves gegl behind.
#[test]
fn a_dependency_nothing_requires_is_an_orphan() {
    let system = System::new(&["core"]);
    let whole = System::new(&["core"]);

    system.install_captured(&[GEGL, MPV]);
    whole.install_captured(&[GIMP, GEGL, MPV]);

    assert_eq!(orphans(&system), Ok(names(&["gegl"])));
    assert_eq!(orphans(&whole), Ok(Vec::new()));
}

#[test]
fn an_explicit_package_is_never_an_orphan() {
    let system = System::new(&["core"]);

    system.install_captured(&[GIMP, YT_DLP]);

    assert_eq!(orphans(&system), Ok(Vec::new()));
}

/// yt-dlp's captured `desc` gets `%REASON%` 1, as if installed for mpv.
#[test]
fn an_optional_dependency_in_use_is_kept() {
    let system = System::new(&["core"]);
    let alone = System::new(&["core"]);

    system.install_captured(&[MPV, YT_DLP]);
    system.append_desc(YT_DLP, "%REASON%\n1\n\n");
    alone.install_captured(&[YT_DLP]);
    alone.append_desc(YT_DLP, "%REASON%\n1\n\n");

    assert_eq!(orphans(&system), Ok(Vec::new()));
    assert_eq!(orphans(&alone), Ok(names(&["yt-dlp"])));
}

#[test]
fn without_a_local_database_orphans_are_unavailable() {
    let system = System::new(&["core"]);

    assert_eq!(orphans(&system), Err(ProbeError::Unavailable));
}
