//! The command bus.
//!
//! Widgets send what the user asked for, and `App::logic` applies it
//! before the next frame. Pages never change the app's state themselves.
//! One list of commands feeds the shortcuts and the palette, so they
//! never disagree.

pub mod boot;
pub mod bootrepair;
pub mod configs;
pub mod console;
mod conversions;
pub mod diagnostics;
pub mod discover;
pub mod locales;
pub mod maintenance;
pub mod network;
pub mod packages;
pub mod shortcuts;
#[cfg(test)]
mod tests;
pub mod users;

use std::panic::Location;

use crossbeam_channel::{Receiver, Sender};
use toolbox_core::Toggle;
use toolbox_core::features::maintenance::RunNow;
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_plan::protocol::TerminalSize;
use toolbox_plan::{
    DnsProvider, GarudaUpdateFlag, PackageName, ShellPath, UnitName, UnitOperation, UnitScope,
    WifiBackend,
};

pub use self::boot::BootCommand;
pub use self::bootrepair::BootRepairCommand;
pub use self::configs::ConfigsCommand;
pub use self::console::ConsoleCommand;
pub use self::diagnostics::DiagnosticsCommand;
pub use self::discover::DiscoverCommand;
pub use self::locales::LocaleCommand;
pub use self::maintenance::MaintenanceCommand;
pub use self::network::NetworkCommand;
pub use self::packages::PackagesCommand;
pub use self::users::{UserEdit, UsersCommand};
use crate::catalogs::CatalogId;
use crate::instance::Arrival;
use crate::page::PageId;
use crate::settings::SettingsCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiCommand {
    Navigate(PageId),
    Apply,
    ToggleSidebar,
    ToggleConsole,
    OpenPalette,
    ReviewChanges,
    /// Removes every queued change. It cannot be taken back, so the app
    /// asks first.
    ClearChanges,
    FocusSearch,
    /// Opens the user guide on the page about the open one. F1, without
    /// Ctrl, as in other desktop apps.
    Help,
    Quit,
}

/// What a page or panel asks of the app's data, as opposed to its window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCommand {
    RemoveChange {
        domain: &'static str,
        key: String,
    },
    /// Bytes for the running program: keys, mouse, terminal replies.
    ConsoleInput(Vec<u8>),
    ConsoleResize(TerminalSize),
    /// Ctrl+Shift+V in the Console: the app reads its clipboard.
    PasteClipboard,
    /// The status bar's Read again, after parts of the system could not
    /// be read.
    ReadSystem,
    CancelPlan,
    /// A click on a catalog entry: queue it on or off, or take the queued
    /// wish back.
    ToggleEntry {
        catalog: CatalogId,
        entry: String,
        wish: Toggle,
    },
    DismissRefusal(CatalogId),
    /// A note about a background result nobody waited on, at the top
    /// right of the page.
    Toast {
        kind: ToastKind,
        text: String,
    },
    Kernel(KernelCommand),
    Home(HomeCommand),
    Tweak(TweakCommand),
    Service(ServiceCommand),
    Discover(DiscoverCommand),
    Setting(SettingsCommand),
    Maintenance(MaintenanceCommand),
    Diagnostics(DiagnosticsCommand),
    Boot(BootCommand),
    Locale(LocaleCommand),
    Network(NetworkCommand),
    Packages(PackagesCommand),
    Users(UsersCommand),
    Configs(ConfigsCommand),
    BootRepair(BootRepairCommand),
    Console(ConsoleCommand),
    /// Another start asked for the window, maybe on a page.
    Activated(Arrival),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceCommand {
    /// Runs at once in the Console.
    Control {
        scope: UnitScope,
        unit: UnitName,
        operation: UnitOperation,
    },
    Logs {
        scope: UnitScope,
        unit: UnitName,
    },
    /// Queues starting the unit at boot (`on`) or not, or takes it back.
    Enable {
        scope: UnitScope,
        unit: UnitName,
        on: bool,
    },
    Mask {
        scope: UnitScope,
        unit: UnitName,
        on: bool,
    },
    Refresh(UnitScope),
}

/// A choice on System Tweaks, queued for the next apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TweakCommand {
    Shell(ShellPath),
    Dns(DnsProvider),
    Wifi(WifiBackend),
    Hblock(bool),
    Flag(GarudaUpdateFlag, bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeCommand {
    RunNow(RunNow),
    /// Queues the system update with the other changes, or takes it back.
    QueueUpgrade(bool),
    Restart,
    StartInstaller,
    Refresh,
    /// The Setup Assistant reached its end, so Home stops offering it.
    SetupFinished,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelCommand {
    Edit(KernelEdit),
    Refresh,
    DismissRefusal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelEdit {
    /// Queues installing (`on`) or removing the kernel with its headers, or
    /// takes that back.
    Toggle {
        kernel: PackageName,
        on: bool,
    },
    Headers {
        kernel: PackageName,
        on: bool,
    },
    Builds {
        kernel: PackageName,
        on: bool,
    },
    AllHeaders,
    AllBuilds,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ui(UiCommand),
    App(AppCommand),
    Confirmed(UiCommand),
    /// A [`UiCommand`] chosen from the keyboard, as a shortcut is. What it
    /// opens or closes lands at once instead of moving.
    Keyed(UiCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sent {
    pub command: Command,
    pub from: &'static Location<'static>,
}

/// The sending end, cloned into every widget that sends.
#[derive(Clone, Debug)]
pub struct CommandSender(Sender<Sent>);

#[derive(Debug)]
pub struct CommandReceiver(Receiver<Sent>);

/// Both ends of the app's bus, made before the window so a start's wish
/// can wait in it.
#[derive(Debug)]
pub struct CommandBus {
    pub sender: CommandSender,
    pub receiver: CommandReceiver,
}

impl UiCommand {
    /// The commands besides page navigation, in palette order.
    pub const ACTIONS: [Self; 7] = [
        Self::Apply,
        Self::ToggleSidebar,
        Self::ToggleConsole,
        Self::OpenPalette,
        Self::FocusSearch,
        Self::Help,
        Self::Quit,
    ];

    pub const fn text_id(self) -> &'static str {
        match self {
            Self::Navigate(page) => page.text_id(),
            Self::Apply => "command-apply",
            Self::ToggleSidebar => "command-toggle-sidebar",
            Self::ToggleConsole => "command-toggle-console",
            Self::OpenPalette => "command-open-palette",
            Self::ReviewChanges => "command-review-changes",
            Self::ClearChanges => "command-clear-changes",
            Self::FocusSearch => "command-focus-search",
            Self::Help => "command-help",
            Self::Quit => "command-quit",
        }
    }

    /// The Fluent id of a line on what the command does. A page's name
    /// says that already.
    pub const fn tooltip(self) -> &'static str {
        match self {
            Self::Navigate(page) => page.text_id(),
            Self::Apply => "command-apply-tooltip",
            Self::ToggleSidebar => "command-toggle-sidebar-tooltip",
            Self::ToggleConsole => "command-toggle-console-tooltip",
            Self::OpenPalette => "command-open-palette-tooltip",
            Self::ReviewChanges => "command-review-changes-tooltip",
            Self::ClearChanges => "command-clear-changes-tooltip",
            Self::FocusSearch => "command-focus-search-tooltip",
            Self::Help => "command-help-tooltip",
            Self::Quit => "command-quit-tooltip",
        }
    }

    /// What the command does cannot be taken back, so the app runs it only
    /// once the user confirmed it.
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::ClearChanges)
    }
}

impl Command {
    /// The command as the log shows it. Console input carries what the
    /// user types, passwords too, and shared output what programs printed,
    /// so only their length is shown.
    pub fn for_log(&self) -> String {
        if let Self::App(AppCommand::ConsoleInput(bytes)) = self {
            return format!("ConsoleInput({} bytes)", bytes.len());
        }

        if let Self::App(AppCommand::Console(ConsoleCommand::Share(text))) = self {
            return format!("Console(Share({} bytes))", text.len());
        }

        format!("{self:?}")
    }
}

impl From<UiCommand> for Command {
    fn from(command: UiCommand) -> Self {
        Self::Ui(command)
    }
}

impl From<AppCommand> for Command {
    fn from(command: AppCommand) -> Self {
        Self::App(command)
    }
}

impl CommandSender {
    /// Remembers the caller, which the app logs with the command.
    #[track_caller]
    pub fn send(&self, command: impl Into<Command>) {
        let sent = Sent {
            command: command.into(),
            from: Location::caller(),
        };

        // An error only means the app is shutting down.
        self.0.send(sent).ok();
    }
}

impl CommandReceiver {
    pub fn drain(&self) -> impl Iterator<Item = Sent> + '_ {
        self.0.try_iter()
    }
}

impl CommandBus {
    #[must_use = "both ends are needed"]
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Unbounded: the UI thread sends and receives itself, so a
/// bounded channel could block the frame on its own queue.
#[must_use = "both ends are needed"]
pub fn channel() -> (CommandSender, CommandReceiver) {
    let (sender, receiver) = crossbeam_channel::unbounded();
    (CommandSender(sender), CommandReceiver(receiver))
}
