#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_exec::sdboot_menu;
use toolbox_plan::protocol::{MAX_LINE_BYTES, ReadKind};
use toolbox_plan::sdboot::SdBootMenu;
use toolbox_plan::{GRUB_DEFAULTS, KernelParameters};
use toolbox_state::boot::{BootLoader, KernelCmdline};

use super::{MenuRead, SystemBootMenu};
use crate::error::ProbeError;
use crate::files;
use crate::pkexec::{self, HelperRun};

const LOADER_INFO: &str =
    "sys/firmware/efi/efivars/LoaderInfo-4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";
const INSTALL_CONF: &str = "etc/kernel/install.conf";
const KERNEL_CMDLINE: &str = "etc/kernel/cmdline";

impl SystemBootMenu {
    /// The ESP is usually root's alone, so this may start the helper. Call
    /// it off the GUI thread and only when the page asks.
    pub(super) fn sdboot_menu(&self, cancel: &Arc<AtomicBool>) -> MenuRead {
        match sdboot_menu(&self.root) {
            Ok(menu) => MenuRead::SdBoot(menu),
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                self.ask_helper_sdboot(cancel)
            }
            Err(error) => MenuRead::Unknown(error.to_string()),
        }
    }

    fn ask_helper_sdboot(&self, cancel: &Arc<AtomicBool>) -> MenuRead {
        let mut command = self.helper.read_command(ReadKind::SdBootMenu);

        match pkexec::ask(&mut command, MAX_LINE_BYTES, cancel) {
            HelperRun::Answered(line) => SdBootMenu::parse(line.trim_end()).map_or_else(
                |_not_a_menu| MenuRead::Unknown("the helper's answer is no menu".to_owned()),
                MenuRead::SdBoot,
            ),
            HelperRun::Refused(refusal) => MenuRead::Refused(refusal),
            HelperRun::Failed(reason) => MenuRead::Unknown(reason),
            HelperRun::Cancelled => MenuRead::Cancelled,
        }
    }
}

pub(super) fn loader(root: &Path) -> BootLoader {
    let info = std::fs::read(root.join(LOADER_INFO)).ok();
    let install_conf = files::read_optional(&root.join(INSTALL_CONF))
        .ok()
        .flatten();

    BootLoader::detect(
        info.as_deref(),
        root.join(GRUB_DEFAULTS).exists(),
        install_conf.as_deref(),
    )
}

pub(super) fn kernel_cmdline(root: &Path) -> Result<KernelCmdline, ProbeError> {
    let text = files::read(&root.join(KERNEL_CMDLINE))?;

    KernelParameters::parse(&text)
        .map(KernelCmdline)
        .ok_or_else(|| {
            ProbeError::Failed(format!(
                "/{KERNEL_CMDLINE} holds a word that is no kernel parameter"
            ))
        })
}
