//! The conversions of the page commands into a bus command, so a page
//! sends its own command type.

use super::{
    AppCommand, Command, DiscoverCommand, HomeCommand, KernelCommand, KernelEdit, ServiceCommand,
    TweakCommand,
};

impl From<KernelCommand> for Command {
    fn from(command: KernelCommand) -> Self {
        Self::App(AppCommand::Kernel(command))
    }
}

impl From<ServiceCommand> for Command {
    fn from(command: ServiceCommand) -> Self {
        Self::App(AppCommand::Service(command))
    }
}

impl From<DiscoverCommand> for Command {
    fn from(command: DiscoverCommand) -> Self {
        Self::App(AppCommand::Discover(command))
    }
}

impl From<TweakCommand> for Command {
    fn from(command: TweakCommand) -> Self {
        Self::App(AppCommand::Tweak(command))
    }
}

impl From<HomeCommand> for Command {
    fn from(command: HomeCommand) -> Self {
        Self::App(AppCommand::Home(command))
    }
}

impl From<KernelEdit> for Command {
    fn from(edit: KernelEdit) -> Self {
        KernelCommand::Edit(edit).into()
    }
}
