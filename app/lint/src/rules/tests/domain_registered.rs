use std::collections::BTreeSet;
use std::path::Path;

use super::super::domain_registered::registered;
use super::super::lint_file_in;

const RULE: &str = "domain-registered";

const REGISTRY: &str = "pub fn all() -> Result<crate::Domains, crate::DuplicateDomain> {
    crate::Domains::default()
        .with::<Packages>()?
        .with::<Hblock>()?
        .with::<crate::features::maintenance::Operations>()
}
";

const PACKAGES: &str = "impl Domain for Packages {}\n";
const LOCALES: &str = "impl crate::Domain for Locales {}\n";

fn count(path: &str, text: &str, domains: Option<&BTreeSet<String>>) -> usize {
    lint_file_in(Path::new(path), text, domains)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .count()
}

#[test]
fn the_registry_names_every_type_it_takes() {
    assert_eq!(
        registered(REGISTRY),
        BTreeSet::from(["Hblock", "Operations", "Packages"].map(str::to_owned))
    );
}

#[test]
fn a_domain_missing_from_the_registry_is_reported() {
    let domains = registered(REGISTRY);

    assert_eq!(
        count(
            "crates/core/src/domains/locales.rs",
            LOCALES,
            Some(&domains)
        ),
        1
    );
    assert_eq!(
        count(
            "crates/core/src/domains/packages.rs",
            PACKAGES,
            Some(&domains)
        ),
        0
    );
}

/// Tests define their own domains, which no registry takes.
#[test]
fn a_test_domain_passes() {
    let domains = registered(REGISTRY);

    assert_eq!(
        count("crates/core/src/planner/tests.rs", LOCALES, Some(&domains)),
        0
    );
}

#[test]
fn without_a_registry_the_rule_is_skipped() {
    assert_eq!(
        count("crates/core/src/domains/locales.rs", LOCALES, None),
        0
    );
}

#[test]
fn other_traits_pass() {
    let domains = registered(REGISTRY);
    let other = "impl Part for Locales {}\nimpl fmt::Display for Locales {}\n";

    assert_eq!(
        count("crates/core/src/domains/locales.rs", other, Some(&domains)),
        0
    );
}
