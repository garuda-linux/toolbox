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

fn count(path: &str, text: &str, domains: &BTreeSet<String>) -> usize {
    lint_file_in(Path::new(path), text, domains)
        .into_iter()
        .filter(|finding| finding.rule == RULE)
        .count()
}

fn names(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn the_registry_names_every_type_it_takes() {
    assert_eq!(
        registered(REGISTRY).unwrap(),
        names(&["Hblock", "Operations", "Packages"])
    );
}

#[test]
fn a_comment_a_string_and_a_test_module_do_not_count() {
    let text = "pub fn all() -> Result<crate::Domains, crate::DuplicateDomain> {
    // .with::<Commented>()?
    let text = \".with::<Quoted>()\";
    crate::Domains::default().with::<Packages>()?
}

fn helper() -> crate::Domains {
    crate::Domains::default().with::<Elsewhere>().unwrap()
}

#[cfg(test)]
mod tests {
    fn all() -> crate::Domains {
        crate::Domains::default().with::<Fake>().unwrap()
    }
}
";

    assert_eq!(registered(text).unwrap(), names(&["Packages"]));
}

#[test]
fn a_file_without_fn_all_is_an_error() {
    registered("fn other() {}\n").unwrap_err();
}

#[test]
fn a_domain_missing_from_the_registry_is_reported() {
    let domains = registered(REGISTRY).unwrap();

    assert_eq!(
        count("crates/core/src/domains/locales.rs", LOCALES, &domains),
        1
    );
    assert_eq!(
        count("crates/core/src/domains/packages.rs", PACKAGES, &domains),
        0
    );
}

/// Tests define their own domains, which no registry takes.
#[test]
fn a_test_domain_passes() {
    let domains = registered(REGISTRY).unwrap();

    assert_eq!(
        count("crates/core/src/planner/tests.rs", LOCALES, &domains),
        0
    );
}

#[test]
fn other_traits_pass() {
    let domains = registered(REGISTRY).unwrap();
    let other = "impl Part for Locales {}\nimpl fmt::Display for Locales {}\n";

    assert_eq!(
        count("crates/core/src/domains/locales.rs", other, &domains),
        0
    );
}
