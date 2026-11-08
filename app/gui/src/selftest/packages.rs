//! The Packages page on the real system: the installed packages, then a
//! search that finds nothing. A machine without pacman's databases shows
//! the page's notice instead, so each wait takes either.

use egui::Key;
use egui::accesskit::Role;

use super::step::Step;

/// What the page says where pacman's databases cannot be read.
const UNAVAILABLE: &str = "The packages are not available on this system: the package databases of pacman could not be read.";

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let [field, nothing] = Step::type_into("Search packages", "zz-no-such-package");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num6),
        Step::WaitContaining("The packages of the repositories"),
        Step::WaitGone("Reading the package lists…"),
        Step::WaitEither("pacman", UNAVAILABLE),
        Step::Shot("packages"),
        field,
        nothing,
        Step::WaitEither("No package matches “zz-no-such-package”", UNAVAILABLE),
        Step::Shot("packages-nothing"),
    ]
}

/// The details of pacman itself on Arch, where its databases exist: one
/// dependency followed and the way back.
#[must_use = "the steps do nothing until a suite runs them"]
pub fn arch_steps() -> Vec<Step> {
    let [field, name] = Step::type_into("Search packages", "pacman");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num6),
        Step::WaitGone("Reading the package lists…"),
        field,
        name,
        Step::Click {
            role: Some(Role::ListItem),
            label: "pacman",
        },
        Step::Wait {
            role: Some(Role::Heading),
            label: "Depends on",
        },
        Step::Shot("packages-detail"),
        Step::Click {
            role: Some(Role::Link),
            label: "bash",
        },
        Step::wait("Back to pacman"),
        Step::Shot("packages-detail-dependency"),
        Step::click("Back to pacman"),
        Step::wait("Back to Packages"),
    ]
}

/// The AUR chip on the dry run's sample list, which `scripts/selftest.sh`
/// turns on in the run's own settings: the list, one package's details
/// and the question before a build, answered with Cancel, so nothing is
/// queued. Runs without pacman's databases too.
#[must_use = "the steps do nothing until a suite runs them"]
pub fn aur_steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num6),
        Step::WaitContaining("The packages of the repositories"),
        Step::WaitGone("Reading the package lists…"),
        // In `all` the packages suite left a search in the field.
        Step::Click {
            role: Some(Role::TextInput),
            label: "Search packages",
        },
        Step::shortcut(Key::A),
        Step::press(Key::Backspace),
        Step::button("AUR"),
        Step::Wait {
            role: Some(Role::ListItem),
            label: "paru-bin",
        },
        Step::Shot("packages-aur"),
        Step::Click {
            role: Some(Role::ListItem),
            label: "paru-bin",
        },
        Step::wait("https://aur.archlinux.org/packages/paru-bin"),
        Step::Shot("packages-aur-detail"),
        Step::click("Back to Packages"),
        Step::wait("Install yay-bin"),
        Step::click("Install yay-bin"),
        Step::wait("Build yay-bin from the AUR?"),
        Step::Shot("packages-aur-install"),
        Step::button("Cancel"),
        Step::WaitGone("Build yay-bin from the AUR?"),
        Step::wait("Install yay-bin"),
    ]
}
