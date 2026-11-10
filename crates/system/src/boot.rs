//! What the Boot options page reads: `/etc/default/grub`, the installed
//! Plymouth themes with the one in use, whether the machine is a virtual
//! one, and the boot menu of `grub.cfg`.
//!
//! The first three are parts of the snapshot. The menu is not: `grub.cfg`
//! is usually root's alone, so reading it may start the root helper, which
//! only happens when the page asks.

#[cfg(test)]
mod tests;

mod sdboot;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::protocol::{MAX_LINE_BYTES, ReadKind};
use toolbox_plan::sdboot::SdBootMenu;
use toolbox_plan::{
    EFI_FIRMWARE, GRUB_CONFIG, GRUB_DEFAULTS, GrubMenu, GrubPlatform, GrubState, PackageName,
    PlymouthTheme,
};
use toolbox_state::boot::{BootLoader, CurrentGrub, KernelCmdline, PlymouthState, VirtualMachine};

use crate::diagnostics::HelperReads;
use crate::error::{ProbeError, from_io};
use crate::pkexec::{self, HelperRun, Refusal};
use crate::{command, files};

const PLYMOUTH_THEMES: &str = "usr/share/plymouth/themes";
/// The administrator's choice, then the distribution's.
const PLYMOUTH_CONFIGS: [&str; 2] = [
    "etc/plymouth/plymouthd.conf",
    "usr/share/plymouth/plymouthd.defaults",
];

const DETECT_VIRT: &str = "systemd-detect-virt";

pub trait BootSource: Send + Sync {
    /// The variables of `/etc/default/grub`.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the file (GRUB is not the boot
    /// loader), [`ProbeError::Failed`] when it cannot be read.
    fn grub(&self) -> Result<CurrentGrub, ProbeError>;

    /// The installed Plymouth themes and the one in use.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without Plymouth's theme folder.
    fn plymouth(&self) -> Result<PlymouthState, ProbeError>;

    /// Whether the system runs in a virtual machine.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `systemd-detect-virt`.
    fn virtual_machine(&self) -> Result<VirtualMachine, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Failed`] when a file that names the loader cannot be
    /// read.
    fn loader(&self) -> Result<BootLoader, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `/etc/kernel/cmdline`.
    fn kernel_cmdline(&self) -> Result<KernelCmdline, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuRead {
    Known(GrubMenu),
    SdBoot(SdBootMenu),
    /// No `grub.cfg`: GRUB never generated a menu here.
    Missing,
    /// pkexec did not run the helper.
    Refused(Refusal),
    /// Why the menu is not known: the helper failed.
    Unknown(String),
    /// The caller set the cancel flag while the helper ran.
    Cancelled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Menu {
    #[default]
    NotAsked,
    Loading,
    Read(MenuRead),
}

impl Menu {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotAsked => "not-asked",
            Self::Loading => "loading",
            Self::Read(MenuRead::Known(_) | MenuRead::SdBoot(_)) => "known",
            Self::Read(MenuRead::Missing) => "missing",
            Self::Read(MenuRead::Refused(refusal)) => refusal.code(),
            Self::Read(MenuRead::Unknown(_) | MenuRead::Cancelled) => "unknown",
        }
    }

    pub const fn known(&self) -> Option<&GrubMenu> {
        match self {
            Self::Read(MenuRead::Known(menu)) => Some(menu),
            Self::NotAsked
            | Self::Loading
            | Self::Read(
                MenuRead::SdBoot(_)
                | MenuRead::Missing
                | MenuRead::Refused(_)
                | MenuRead::Unknown(_)
                | MenuRead::Cancelled,
            ) => None,
        }
    }

    pub const fn sdboot(&self) -> Option<&SdBootMenu> {
        match self {
            Self::Read(MenuRead::SdBoot(menu)) => Some(menu),
            Self::NotAsked
            | Self::Loading
            | Self::Read(
                MenuRead::Known(_)
                | MenuRead::Missing
                | MenuRead::Refused(_)
                | MenuRead::Unknown(_)
                | MenuRead::Cancelled,
            ) => None,
        }
    }

    /// The id of `kernel`'s menu entry, `""` while unknown.
    pub fn kernel_entry(&self, kernel: &PackageName) -> &str {
        self.known()
            .and_then(|menu| menu.kernel_entry(kernel))
            .map_or("", |entry| entry.id.as_str())
    }
}

/// Reads the boot menu. Blocks and may start the root helper, so call it
/// off the GUI thread and only when the page asks.
pub trait BootMenuSource: Send + Sync {
    /// The menu. A read through the helper waits for the password dialog
    /// until `cancel` is set.
    fn menu(&self, cancel: &Arc<AtomicBool>) -> MenuRead;
}

/// [`BootSource`] from the files below a root folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemBoot {
    root: PathBuf,
    detect_virt: PathBuf,
}

/// [`BootMenuSource`]: the app reads `grub.cfg` itself where the user
/// may, and asks the helper's `read grub-menu` where the system says no.
pub struct SystemBootMenu {
    root: PathBuf,
    helper: Arc<dyn HelperReads>,
}

impl SystemBoot {
    /// The running system.
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/", DETECT_VIRT)
    }

    /// Another root folder and detection program, for tests.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(root: impl Into<PathBuf>, detect_virt: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            detect_virt: detect_virt.into(),
        }
    }

    /// The theme a Plymouth config names, when it is installed.
    fn named_theme(&self, config: &str, installed: &[PlymouthTheme]) -> Option<PlymouthTheme> {
        files::read_optional(&self.root.join(config))
            .ok()
            .flatten()
            .and_then(|text| theme_of(&text))
            .filter(|theme| installed.contains(theme))
    }
}

impl BootSource for SystemBoot {
    fn grub(&self) -> Result<CurrentGrub, ProbeError> {
        files::read(&self.root.join(GRUB_DEFAULTS)).map(|text| CurrentGrub(GrubState::read(&text)))
    }

    /// A theme is a folder with a `<name>.plymouth` file of its name, as
    /// `plymouth-set-default-theme --list` takes it.
    fn plymouth(&self) -> Result<PlymouthState, ProbeError> {
        let folder = self.root.join(PLYMOUTH_THEMES);
        let entries =
            std::fs::read_dir(&folder).map_err(|error| from_io(folder.display(), error))?;

        let mut themes: Vec<PlymouthTheme> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;

                entry
                    .path()
                    .join(format!("{name}.plymouth"))
                    .is_file()
                    .then_some(name)
            })
            .filter_map(|name| PlymouthTheme::new(name).ok())
            .collect();

        themes.sort();

        let current = PLYMOUTH_CONFIGS
            .into_iter()
            .find_map(|config| self.named_theme(config, &themes));

        Ok(PlymouthState { themes, current })
    }

    /// `--vm` leaves containers out: it exits with 0 and names the
    /// hypervisor only in a virtual machine.
    fn virtual_machine(&self) -> Result<VirtualMachine, ProbeError> {
        let finished = command::run(self.detect_virt.as_os_str(), &["--vm"])?;
        let named = !matches!(finished.stdout.trim(), "" | "none");

        Ok(VirtualMachine(finished.status.success() && named))
    }

    fn loader(&self) -> Result<BootLoader, ProbeError> {
        sdboot::loader(&self.root)
    }

    fn kernel_cmdline(&self) -> Result<KernelCmdline, ProbeError> {
        sdboot::kernel_cmdline(&self.root)
    }
}

impl SystemBootMenu {
    /// The running system.
    #[must_use = "the adapter does nothing until asked"]
    pub fn system(helper: Arc<dyn HelperReads>) -> Self {
        Self::new(Path::new("/"), helper)
    }

    /// Another root folder, for tests.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(root: &Path, helper: Arc<dyn HelperReads>) -> Self {
        Self {
            root: root.to_owned(),
            helper,
        }
    }

    /// The helper's answer is decoded into the same type it was made
    /// from, so anything but a menu is refused whole.
    fn ask_helper(&self, cancel: &Arc<AtomicBool>) -> MenuRead {
        let mut command = self.helper.read_command(ReadKind::GrubMenu);

        match pkexec::ask(&mut command, MAX_LINE_BYTES, cancel) {
            HelperRun::Answered(line) => GrubMenu::parse(line.trim_end()).map_or_else(
                |_not_a_menu| MenuRead::Unknown("the helper's answer is no menu".to_owned()),
                MenuRead::Known,
            ),
            HelperRun::Refused(refusal) => MenuRead::Refused(refusal),
            HelperRun::Failed(reason) => MenuRead::Unknown(reason),
            HelperRun::Cancelled => MenuRead::Cancelled,
        }
    }
}

impl BootMenuSource for SystemBootMenu {
    fn menu(&self, cancel: &Arc<AtomicBool>) -> MenuRead {
        if sdboot::loader(&self.root) == Ok(BootLoader::SystemdBoot) {
            return self.sdboot_menu(cancel);
        }

        match files::read_lossy(&self.root.join(GRUB_CONFIG)) {
            Ok(config) => {
                MenuRead::Known(GrubMenu::for_platform(&config, grub_platform(&self.root)))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => MenuRead::Missing,
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                self.ask_helper(cancel)
            }
            Err(error) => MenuRead::Unknown(error.to_string()),
        }
    }
}

/// The firmware the system below `root` started on, which picks the
/// branches of `grub.cfg` GRUB shows.
#[must_use = "it only reads the firmware folder"]
pub fn grub_platform(root: &Path) -> GrubPlatform {
    GrubPlatform::from_efi_firmware(root.join(EFI_FIRMWARE).is_dir())
}

/// The `Theme=` of a Plymouth config, whatever its section.
fn theme_of(config: &str) -> Option<PlymouthTheme> {
    let name = config
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;

            (key.trim() == "Theme").then(|| value.trim().to_owned())
        })?;

    PlymouthTheme::new(name).ok()
}
