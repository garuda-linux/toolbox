//! The services behind the pages, which App feeds each frame.
//!
//! App also tells them each end of a plan and does what they answer. A new
//! service is a field here, its line in [`Services::all_mut`] and its
//! [`Provides`] impl.

mod adapters;

use toolbox_core::features::boot::edit::Edit;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::Plan;

use crate::artwork::Artwork;
use crate::boot::Boot;
use crate::bootrepair::BootRepair;
use crate::command::AppCommand;
use crate::configs::Configs;
use crate::diagnostics::Diagnostics;
use crate::home::Home;
use crate::kernels::Kernels;
use crate::locales::Locales;
use crate::maintenance::Maintenance;
use crate::network::Network;
use crate::packages::Packages;
use crate::page::PageId;
use crate::toggle_pages::TogglePages;
use crate::units::Units;
use crate::users::Users;

/// What a service asks of App once its work came in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Runs at once in the Console, unless another plan runs.
    RunPlan(Plan),
    /// A part of the system came in: the snapshot is joined again and the
    /// queue meets it.
    Recombine,
    EditBoot(Edit),
    /// The Config Files page's next step, see [`crate::configs::Act`].
    Configs(crate::configs::Act),
    /// The edit needs the user on the page, which the Console would cover.
    CloseConsole,
    /// Copied to the clipboard when the settings ask for it, with the toast
    /// of this text id.
    Copy {
        text: String,
        toast: &'static str,
    },
    /// Whether the sidebar lists Boot Repair.
    OfferBootRepair(bool),
    /// A page asks for another one, as Boot options for a kernel the
    /// Kernels page proposes.
    ShowPage(PageId),
}

/// What a service made of a command: the queue edit of a click, which App
/// takes in the same frame, and what App should do next.
#[derive(Debug, Default)]
pub struct Handled {
    /// A service took the command, so App does nothing more with it, also
    /// for a click that was refused.
    pub taken: bool,
    pub edit: Option<ChangeSet>,
    pub answers: Vec<AppCommand>,
}

impl Handled {
    pub const fn taken() -> Self {
        Self {
            taken: true,
            edit: None,
            answers: Vec::new(),
        }
    }

    pub const fn edit(edit: ChangeSet) -> Self {
        Self {
            taken: true,
            edit: Some(edit),
            answers: Vec::new(),
        }
    }

    pub const fn answers(answers: Vec<AppCommand>) -> Self {
        Self {
            taken: true,
            edit: None,
            answers,
        }
    }
}

/// A page's work off the frame. Each hook answers with what App should
/// do next, so a service needs nothing of App.
pub trait PageService {
    /// A command of the page. `changes` is the queue the click saw, which
    /// the service never changes itself: it answers with the edit.
    fn handle(
        &mut self,
        _command: &AppCommand,
        _changes: &ChangeSet,
        _snapshot: &Snapshot,
    ) -> Handled {
        Handled::default()
    }

    /// Drops the last refused click of `page`. Returns whether this
    /// service keeps that page's refusals.
    fn dismiss_refusal(&mut self, _page: PageId) -> bool {
        false
    }

    /// What App hands every command to. Each page dismisses its refusal
    /// the same way, so only [`PageService::handle`] differs.
    fn receive(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        if let AppCommand::DismissRefusal(page) = command {
            return Handled {
                taken: self.dismiss_refusal(*page),
                ..Handled::default()
            };
        }

        self.handle(command, changes, snapshot)
    }

    /// The results that came in since the last frame. `page` is the page
    /// in view, which a service may start reading for.
    fn take(&mut self, _page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        Vec::new()
    }

    /// A plan began to run, the one this service sent if it waits.
    fn plan_started(&mut self) -> Vec<AppCommand> {
        Vec::new()
    }

    /// Another plan was running, so the one sent never started.
    fn plan_refused(&mut self) -> Vec<AppCommand> {
        Vec::new()
    }

    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        Vec::new()
    }
}

#[derive(Debug, Default)]
pub struct Services {
    pub home: Home,
    pub maintenance: Maintenance,
    pub diagnostics: Diagnostics,
    pub boot: Boot,
    pub network: Network,
    pub packages: Packages,
    pub users: Users,
    pub configs: Configs,
    pub boot_repair: BootRepair,
    pub kernels: Kernels,
    pub locales: Locales,
    pub toggle_pages: TogglePages,
    pub units: Units,
    /// Only pages read it: the pictures load as rows ask, beside the
    /// frame, so App feeds it nothing.
    pub artwork: Artwork,
}

impl Services {
    /// Every service, in the order App feeds them.
    pub fn all_mut(&mut self) -> [&mut dyn PageService; 13] {
        [
            &mut self.home,
            &mut self.maintenance,
            &mut self.diagnostics,
            &mut self.boot,
            &mut self.network,
            &mut self.packages,
            &mut self.users,
            &mut self.configs,
            &mut self.boot_repair,
            &mut self.kernels,
            &mut self.locales,
            &mut self.toggle_pages,
            &mut self.units,
        ]
    }

    /// The service of type `T`, as a page asks for its own.
    pub fn service<T>(&self) -> &T
    where
        Self: Provides<T>,
    {
        self.provide()
    }
}

/// Which service types [`Services`] holds, so a lookup of one it lacks
/// does not compile.
pub trait Provides<T> {
    fn provide(&self) -> &T;
}

impl Provides<Maintenance> for Services {
    fn provide(&self) -> &Maintenance {
        &self.maintenance
    }
}

impl Provides<Diagnostics> for Services {
    fn provide(&self) -> &Diagnostics {
        &self.diagnostics
    }
}

impl Provides<Boot> for Services {
    fn provide(&self) -> &Boot {
        &self.boot
    }
}

impl Provides<Network> for Services {
    fn provide(&self) -> &Network {
        &self.network
    }
}

impl Provides<Packages> for Services {
    fn provide(&self) -> &Packages {
        &self.packages
    }
}

impl Provides<Users> for Services {
    fn provide(&self) -> &Users {
        &self.users
    }
}

impl Provides<Configs> for Services {
    fn provide(&self) -> &Configs {
        &self.configs
    }
}

impl Provides<BootRepair> for Services {
    fn provide(&self) -> &BootRepair {
        &self.boot_repair
    }
}

impl Provides<Kernels> for Services {
    fn provide(&self) -> &Kernels {
        &self.kernels
    }
}

impl Provides<Locales> for Services {
    fn provide(&self) -> &Locales {
        &self.locales
    }
}

impl Provides<TogglePages> for Services {
    fn provide(&self) -> &TogglePages {
        &self.toggle_pages
    }
}

impl Provides<Units> for Services {
    fn provide(&self) -> &Units {
        &self.units
    }
}

impl Provides<Artwork> for Services {
    fn provide(&self) -> &Artwork {
        &self.artwork
    }
}
