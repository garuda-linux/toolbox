#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_exec::{Environment, locate_program, sdboot_menu};
use toolbox_plan::procedure::uki_rebuilds;
use toolbox_plan::protocol::{ProgramAnswer, ReadKind};
use toolbox_plan::sdboot::{KERNEL_CMDLINE, SdBootMenu, bootctl_default};
use toolbox_plan::{GRUB_DEFAULTS, LoaderEntryId};
use toolbox_state::boot::{BootLoader, KernelCmdline};

use super::{MenuRead, SystemBootMenu};
use crate::diagnostics::HelperReads;
use crate::error::{ProbeError, from_io};
use crate::pkexec::{self, HelperRun};
use crate::{command, files};

const LOADER_INFO: &str =
    "sys/firmware/efi/efivars/LoaderInfo-4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";
const INSTALL_CONF: &str = "etc/kernel/install.conf";
const PROC_CMDLINE: &str = "proc/cmdline";

impl SystemBootMenu {
    /// The ESP is usually root's alone, so this may start the helper. Call
    /// it off the GUI thread and only when the page asks.
    pub(super) fn sdboot_menu(&self, cancel: &Arc<AtomicBool>) -> MenuRead {
        match sdboot_menu(&self.root) {
            Ok(mut menu) => {
                menu.default = default_as_user();
                MenuRead::SdBoot(menu)
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                match self.ask_helper(
                    ReadKind::SdBootMenu,
                    cancel,
                    SdBootMenu::parse,
                    MenuRead::SdBoot,
                ) {
                    MenuRead::SdBoot(mut menu) => {
                        menu.default = default_through_helper(self.helper.as_ref(), cancel);
                        MenuRead::SdBoot(menu)
                    }
                    other @ (MenuRead::Known(_)
                    | MenuRead::Missing
                    | MenuRead::Refused(_)
                    | MenuRead::Unknown(_)
                    | MenuRead::Unreadable(_)
                    | MenuRead::Cancelled) => other,
                }
            }
            Err(error) => MenuRead::Unreadable(error.kind()),
        }
    }
}

/// bootctl's default, run as the user where the ESP is readable. A missing
/// or failing bootctl leaves the default unknown.
fn default_as_user() -> Option<LoaderEntryId> {
    let (program, arguments) = ReadKind::SdBootDefault.program()?;
    let path = locate_program(program, Environment::Inherited);
    let finished = command::run(path.as_os_str(), arguments).ok()?;

    finished
        .status
        .success()
        .then(|| bootctl_default(&finished.stdout))
        .flatten()
}

/// bootctl's default through the helper, where the ESP is root's alone.
/// Another answer than a run with exit code 0 leaves the default unknown.
fn default_through_helper(
    helper: &dyn HelperReads,
    cancel: &Arc<AtomicBool>,
) -> Option<LoaderEntryId> {
    let mut command = helper.read_command(ReadKind::SdBootDefault);

    match pkexec::ask(&mut command, ProgramAnswer::MAX_LINE_BYTES, cancel) {
        HelperRun::Answered(line) => match ProgramAnswer::parse(line.trim_end()) {
            Ok(ProgramAnswer::Ran {
                output,
                errors: _,
                code,
                truncated: _,
            }) => {
                if code == Some(0_i32) {
                    bootctl_default(&output)
                } else {
                    None
                }
            }
            Ok(ProgramAnswer::Missing) | Err(_) => None,
        },
        HelperRun::Refused(_) | HelperRun::Failed(_) | HelperRun::Cancelled => None,
    }
}

/// # Errors
///
/// A failed [`ProbeError`] when `LoaderInfo` or `install.conf` exists but
/// cannot be read: a guess would show the wrong loader's page.
pub(super) fn loader(root: &Path) -> Result<BootLoader, ProbeError> {
    let info_path = root.join(LOADER_INFO);
    let info = match std::fs::read(&info_path) {
        Ok(info) => Some(info),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(from_io(info_path.display(), error)),
    };

    let install_conf = files::read_optional(&root.join(INSTALL_CONF))?;

    Ok(BootLoader::detect(
        info.as_deref(),
        root.join(GRUB_DEFAULTS).exists(),
        install_conf.as_deref(),
    ))
}

pub(super) fn kernel_cmdline(root: &Path) -> Result<KernelCmdline, ProbeError> {
    if let Some(text) = files::read_optional(&root.join(KERNEL_CMDLINE))? {
        return Ok(KernelCmdline::read(&text, false));
    }

    // kernel-install takes the running line, other builders such as
    // NixOS' write their entries without the file.
    let builds = uki_rebuilds()
        .iter()
        .any(|command| command.program().below(root).is_file());

    if !builds {
        return Err(ProbeError::Unavailable);
    }

    files::read(&root.join(PROC_CMDLINE)).map(|text| KernelCmdline::read(&text, true))
}
