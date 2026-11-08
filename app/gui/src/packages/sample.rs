//! The AUR of a dry run: three fixed packages, so the self-test and a
//! look at the page never reach the AUR.

use toolbox_aur::{AurError, AurSource};
use toolbox_core::features::packages::aur::{AurIndex, AurPackage};
use toolbox_plan::PackageName;

/// 2025-01-10, so dates on the page read the same on every run.
const DAY: i64 = 1_736_467_200;

pub struct SampleAur;

impl AurSource for SampleAur {
    fn index(&self) -> Result<AurIndex, AurError> {
        let packages = [
            ("paru-bin", "Feature packed AUR helper", 30.5_f64, None),
            (
                "yay-bin",
                "Yet another yogurt, an AUR helper",
                12.25_f64,
                None,
            ),
            (
                "toolbox-sample-git",
                "A package flagged out of date",
                0.5_f64,
                Some(DAY),
            ),
        ]
        .into_iter()
        .filter_map(|(name, description, popularity, out_of_date)| {
            Some(AurPackage {
                name: PackageName::new(name).ok()?,
                version: "1.0-1".to_owned(),
                description: description.to_owned(),
                url: Some("https://example.org/".to_owned()),
                votes: 42,
                popularity,
                out_of_date,
                maintainer: Some("sample".to_owned()),
                first_submitted: DAY,
                last_modified: DAY,
                depends: vec!["glibc".to_owned()],
                make_depends: vec!["cargo".to_owned()],
                opt_depends: vec!["git: for -git packages".to_owned()],
                provides: Vec::new(),
                conflicts: Vec::new(),
                licenses: vec!["GPL-3.0-or-later".to_owned()],
                keywords: vec!["helper".to_owned()],
            })
        })
        .collect();

        Ok(AurIndex {
            packages,
            downloaded: DAY,
        })
    }
}
