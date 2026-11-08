//! A click on the Network page.

use toolbox_core::features::network::RadioTarget;
use toolbox_plan::{HopLimit, HostName, KernelModule, PingCount, PingTimeout};

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkCommand {
    /// Reads the system, the page's lists and the internet again.
    Refresh,
    /// Blocks (`blocked`) or unblocks a radio, at once and without a
    /// password.
    SetRadio {
        target: RadioTarget,
        blocked: bool,
    },
    /// Asks the address service once.
    LookupAddress,
    ForgetAddress,
    /// Starts one of the page's services in the Console.
    StartService(&'static str),
    /// Loads (`load`) or unloads a driver in the Console.
    LoadDriver {
        module: KernelModule,
        load: bool,
    },
    /// Queues the driver for the app's blacklist file (`on`), or off it.
    Blacklist {
        module: KernelModule,
        on: bool,
    },
    Ping {
        host: HostName,
        count: PingCount,
        timeout: PingTimeout,
    },
    Traceroute {
        host: HostName,
        hops: HopLimit,
    },
    /// Queues the hotspot tool, or takes it back.
    ToggleHotspot,
    OpenHotspot,
    DismissRefusal,
}

impl From<NetworkCommand> for Command {
    fn from(command: NetworkCommand) -> Self {
        Self::App(AppCommand::Network(command))
    }
}
