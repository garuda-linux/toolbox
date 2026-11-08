//! The `if` blocks of `grub.cfg` that test `$grub_platform`. GRUB runs
//! only the branch of the firmware it started on, so an entry in another
//! branch is not in its menu and takes no position.

use super::words;

const PLATFORM_VARIABLES: [&str; 2] = ["$grub_platform", "${grub_platform}"];

/// The firmware GRUB runs on, as `$grub_platform` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrubPlatform {
    Efi,
    /// A BIOS machine.
    Pc,
}

impl GrubPlatform {
    /// `efi` is whether `/sys/firmware/efi` exists, which the kernel makes
    /// only when UEFI started the system.
    pub const fn from_efi_firmware(efi: bool) -> Self {
        if efi { Self::Efi } else { Self::Pc }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Efi => "efi",
            Self::Pc => "pc",
        }
    }
}

/// One `if` whose `fi` has not come yet.
struct Branch {
    /// Whether any of its conditions tests the platform. The others are
    /// unknown here, so all their branches count.
    platform: bool,
    /// Whether a platform branch before this one matched.
    taken: bool,
    active: bool,
}

pub(super) struct Branches {
    platform: GrubPlatform,
    open: Vec<Branch>,
}

impl Branches {
    pub(super) const fn new(platform: GrubPlatform) -> Self {
        Self {
            platform,
            open: Vec::new(),
        }
    }

    /// Whether GRUB runs the lines read now.
    pub(super) fn active(&self) -> bool {
        self.open.iter().all(|branch| branch.active)
    }

    pub(super) fn line(&mut self, line: &str) {
        if let Some(condition) = line.strip_prefix("if ") {
            let matched = self.matches(condition);

            self.open.push(Branch {
                platform: matched.is_some(),
                taken: matched == Some(true),
                active: matched != Some(false),
            });
        } else if let Some(condition) = line.strip_prefix("elif ") {
            let matched = self.matches(condition);

            if let Some(branch) = self.open.last_mut() {
                branch.platform |= matched.is_some();
                branch.active = !branch.taken && matched != Some(false);
                branch.taken |= matched == Some(true);
            }
        } else if line == "else" {
            if let Some(branch) = self.open.last_mut() {
                branch.active = !branch.platform || !branch.taken;
            }
        } else if line == "fi" {
            self.open.pop();
        } else {
            // Other lines open and close no branch.
        }
    }

    /// `None` when the condition tests anything but the platform.
    fn matches(&self, condition: &str) -> Option<bool> {
        let words = words(condition);
        let [open, left, operator, right, ..] = words.as_slice() else {
            return None;
        };

        if open != "[" {
            return None;
        }

        let equal = match operator.as_str() {
            "=" | "==" => true,
            "!=" => false,
            _ => return None,
        };

        // `x$grub_platform = xefi` keeps an empty variable from breaking the test.
        let (left, right) = match (left.strip_prefix('x'), right.strip_prefix('x')) {
            (Some(left), Some(right)) => (left, right),
            _ => (left.as_str(), right.as_str()),
        };

        let value = if PLATFORM_VARIABLES.contains(&left) {
            right
        } else if PLATFORM_VARIABLES.contains(&right) {
            left
        } else {
            return None;
        };

        Some((value == self.platform.as_str()) == equal)
    }
}
