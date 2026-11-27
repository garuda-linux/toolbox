//! The AUR's web page of a package, as a link for the window and the CLI.

use toolbox_plan::PackageName;

const PACKAGES_PAGE: &str = "https://aur.archlinux.org/packages/";

/// `https://aur.archlinux.org/packages/<name>`. A [`PackageName`] holds
/// no `/`, `?` or `#`, so the link stays on that page.
pub fn package_page(name: &PackageName) -> String {
    format!("{PACKAGES_PAGE}{name}")
}
