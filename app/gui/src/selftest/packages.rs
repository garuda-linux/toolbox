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
    // On Arch pacman stands far down the installed list, out of the window,
    // until a search brings it up.
    let [field, pacman] = Step::type_into("Search packages", "pacman");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num6),
        Step::WaitContaining("The packages of the repositories"),
        Step::WaitGone("Reading the package lists…"),
        field,
        pacman,
        Step::WaitEither("pacman", UNAVAILABLE),
        Step::Shot("packages"),
        Step::shortcut(Key::A),
        Step::Text("zz-no-such-package"),
        Step::WaitEither("Nothing matches “zz-no-such-package”", UNAVAILABLE),
        Step::Shot("packages-nothing"),
    ]
}

/// The details of pacman itself on Arch, where its databases exist: one
/// dependency followed, the way back and the optional-only dependencies.
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
        Step::press(Key::Escape),
        Step::Wait {
            role: Some(Role::ListItem),
            label: "pacman",
        },
        Step::Shot("packages-closed"),
        Step::shortcut(Key::A),
        Step::press(Key::Backspace),
        Step::button("Optional"),
        Step::Shot("packages-optional"),
    ]
}

/// The AUR chip on the dry run's sample list. Cancel answers the build
/// question, so nothing is queued.
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
        Step::Click {
            role: Some(Role::ListItem),
            label: "yay-bin",
        },
        Step::wait("Install yay-bin"),
        Step::click("Install yay-bin"),
        Step::wait("Build yay-bin from the AUR?"),
        Step::Shot("packages-aur-install"),
        Step::button("Cancel"),
        Step::WaitGone("Build yay-bin from the AUR?"),
        Step::Click {
            role: Some(Role::ListItem),
            label: "yay-bin",
        },
        Step::wait("Install yay-bin"),
    ]
}

/// Queued AUR builds whose orders the sample source answers: octopi-git gets
/// its order and shows what it builds, toolbox-sample-git has none and holds
/// Apply with the reason. A dry run skips the user steps, so the review that
/// waits for Continue is not reached here.
pub fn aur_orders_steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num6),
        Step::WaitContaining("The packages of the repositories"),
        Step::WaitGone("Reading the package lists…"),
        Step::Click {
            role: Some(Role::TextInput),
            label: "Search packages",
        },
        Step::shortcut(Key::A),
        Step::press(Key::Backspace),
        Step::button("AUR"),
        Step::Click {
            role: Some(Role::ListItem),
            label: "octopi-git",
        },
        Step::wait("Install octopi-git"),
        Step::click("Install octopi-git"),
        Step::wait("Build octopi-git from the AUR?"),
        Step::button("Build and Install"),
        Step::WaitGone("Build octopi-git from the AUR?"),
        Step::click("Review"),
        Step::WaitContaining("Builds alpm_octopi_utils-git, octopi-git."),
        Step::Shot("packages-aur-orders-ready"),
        Step::Click {
            role: Some(Role::ListItem),
            label: "toolbox-sample-git",
        },
        Step::wait("Install toolbox-sample-git"),
        Step::click("Install toolbox-sample-git"),
        Step::wait("Build toolbox-sample-git from the AUR?"),
        Step::button("Build and Install"),
        Step::WaitGone("Build toolbox-sample-git from the AUR?"),
        Step::WaitContaining("toolbox-sample-git has no build order"),
        Step::Shot("packages-aur-orders-missing"),
    ]
}
