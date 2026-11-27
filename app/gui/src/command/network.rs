//! A click on the Network page.

use toolbox_core::features::network::{NetworkUnit, RadioTarget};
use toolbox_plan::{HopLimit, HostName, KernelModule, PingCount, PingTimeout, Toggle};

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
    StartService(NetworkUnit),
    /// Loads (`load`) or unloads a driver in the Console.
    LoadDriver {
        module: KernelModule,
        load: bool,
    },
    /// Queues the driver for the app's blacklist file (`on`), or off it.
    Blacklist {
        module: KernelModule,
        on: Toggle,
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
    /// Queues pciutils, the tool that lists the hardware, or takes it
    /// back. A refusal stays on the page.
    TogglePciutils {
        wish: Toggle,
    },
}

impl From<NetworkCommand> for Command {
    fn from(command: NetworkCommand) -> Self {
        Self::App(AppCommand::Network(command))
    }
}
