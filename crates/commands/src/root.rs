//! The command line's way to the root helper: pkexec as in the window, and
//! sudo when pkexec cannot ask (no polkit agent, not setuid root), since a
//! terminal is there to type the password in. Never after pkexec asked and
//! was refused, so nobody is asked twice.

use std::sync::Arc;

use tokio::process::Command;
use toolbox_cli::confirm::Prompt as _;
use toolbox_plan::protocol::Mode;
use toolbox_services::apply::{HelperLauncher, Launcher};

use super::out::{Settings, warn};
use super::terminal::Terminal;

pub const SUDO_NOTE: &str = "pkexec is not available, asking through sudo";

struct CommandLine(HelperLauncher);

impl Launcher for CommandLine {
    fn command(&self) -> Command {
        self.0.command()
    }

    fn mode(&self) -> Mode {
        self.0.mode()
    }

    fn inhibitor(&self) -> toolbox_services::apply::Inhibitor {
        self.0.inhibitor()
    }

    fn fallback(&self) -> Option<Arc<dyn Launcher>> {
        let sudo = self.0.through_sudo()?;
        Some(Arc::new(sudo))
    }
}

/// How a read reaches the root helper: sudo at once on a terminal when pkexec
/// cannot raise at all, else pkexec.
pub fn helper(settings: Settings) -> HelperLauncher {
    let pkexec = HelperLauncher::from_process(toolbox_services::i18n::FALLBACK);
    if pkexec.pkexec_usable()
        || !(Terminal {
            color: settings.color,
        })
        .is_terminal()
    {
        return pkexec;
    }

    pkexec.through_sudo().map_or(pkexec, |sudo| {
        warn(settings, SUDO_NOTE);
        sudo
    })
}

/// How a change reaches the root helper: sudo after a run where pkexec could
/// not ask.
pub fn launcher(settings: Settings) -> Arc<dyn Launcher> {
    Arc::new(CommandLine(helper(settings)))
}
