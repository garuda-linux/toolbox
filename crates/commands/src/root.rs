//! The command line's way to the root helper: pkexec as in the window, and
//! sudo when pkexec cannot ask (no polkit agent, not setuid root), since a
//! terminal is there to type the password in. Never after pkexec asked and
//! was refused, so nobody is asked twice.

use std::io::{IsTerminal as _, Write as _};
use std::sync::Arc;

use tokio::process::Command;
use toolbox_cli::confirm::Prompt as _;
use toolbox_cli::output::{ColorMode, Paint};
use toolbox_plan::protocol::Mode;
use toolbox_services::apply::{HelperLauncher, Launcher};

use super::out::style_on;
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
pub fn helper(color: ColorMode) -> HelperLauncher {
    let pkexec = HelperLauncher::from_process();
    if pkexec.pkexec_usable() || !(Terminal { color }).is_terminal() {
        return pkexec;
    }

    pkexec.through_sudo().map_or(pkexec, |sudo| {
        // A note beside the command's own output, which goes to stdout.
        let paint = Paint::new(style_on(color, std::io::stderr().is_terminal()));
        writeln!(std::io::stderr(), "{}", paint.warning(SUDO_NOTE)).ok();
        sudo
    })
}

/// How a change reaches the root helper: sudo after a run where pkexec could
/// not ask.
pub fn launcher(color: ColorMode) -> Arc<dyn Launcher> {
    Arc::new(CommandLine(helper(color)))
}
