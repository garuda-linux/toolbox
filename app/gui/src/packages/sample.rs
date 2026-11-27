//! The AUR of a self-test build's dry run: three fixed packages, so the
//! self-test never reaches the AUR. Release builds leave them out.

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
                version: toolbox_text::plain("1.0-1"),
                description: toolbox_text::plain(description),
                url: Some(toolbox_text::plain("https://example.org/")),
                votes: 42,
                popularity,
                out_of_date,
                maintainer: Some(toolbox_text::plain("sample")),
                first_submitted: DAY,
                last_modified: DAY,
                depends: vec![toolbox_text::plain("glibc")],
                make_depends: vec![toolbox_text::plain("cargo")],
                opt_depends: vec![toolbox_text::plain("git: for -git packages")],
                provides: Vec::new(),
                conflicts: Vec::new(),
                licenses: vec![toolbox_text::plain("GPL-3.0-or-later")],
                keywords: vec![toolbox_text::plain("helper")],
            })
        })
        .collect();

        Ok(AurIndex {
            packages,
            downloaded: DAY,
            stale: false,
        })
    }
}
