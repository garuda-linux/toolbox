//! The names of the kernel packages: which are base kernels, their headers,
//! the repository a kernel is taken from and the flavour a running release
//! names.

use toolbox_plan::PackageName;

pub(super) const PREFIX: &str = "linux";
const HEADERS: &str = "-headers";

/// The repository pacman takes a kernel from when several offer it, on a
/// default Garuda installation. Repositories not listed come last, in the
/// order of `pacman.conf`.
const REPO_PRIORITY: [&str; 4] = ["garuda", "core", "extra", "chaotic-aur"];

pub(super) fn is_base(name: &PackageName) -> bool {
    name.as_str().starts_with(PREFIX) && !name.as_str().ends_with(HEADERS)
}

pub(super) fn headers_of(name: &PackageName) -> Option<PackageName> {
    PackageName::new(format!("{name}{HEADERS}")).ok()
}

pub(super) fn priority(repo: &str) -> usize {
    REPO_PRIORITY
        .iter()
        .position(|known| *known == repo)
        .unwrap_or(REPO_PRIORITY.len())
}

pub(super) fn flavour_match(release: &str, name: &PackageName) -> Option<usize> {
    let flavour = name
        .as_str()
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_prefix('-'));

    flavour.map_or_else(
        || (name.as_str() == PREFIX && release.contains("-arch")).then_some(0),
        |flavour| {
            release
                .ends_with(&format!("-{flavour}"))
                .then_some(flavour.len())
        },
    )
}
