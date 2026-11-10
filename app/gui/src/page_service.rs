//! The services behind the pages, which App feeds each frame.
//!
//! App also tells them each end of a plan and does what they answer. A new
//! service is a field here, its line in [`Services::all_mut`] and its
//! [`Provides`] impl.

mod adapters;

use toolbox_core::Snapshot;
use toolbox_core::features::boot::edit::Edit;
use toolbox_plan::Plan;

use crate::boot::Boot;
use crate::bootrepair::BootRepair;
use crate::command::AppCommand;
use crate::configs::Configs;
use crate::diagnostics::Diagnostics;
use crate::maintenance::Maintenance;
use crate::network::Network;
use crate::packages::Packages;
use crate::page::PageId;
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
    /// Boot Repair shows but is no longer offered.
    LeaveBootRepair,
}

/// A page's work off the frame. Each hook answers with what App should
/// do next, so a service needs nothing of App.
pub trait PageService {
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
    pub maintenance: Maintenance,
    pub diagnostics: Diagnostics,
    pub boot: Boot,
    pub network: Network,
    pub packages: Packages,
    pub users: Users,
    pub configs: Configs,
    pub boot_repair: BootRepair,
}

impl Services {
    /// Every service, in the order App feeds them.
    pub fn all_mut(&mut self) -> [&mut dyn PageService; 8] {
        [
            &mut self.maintenance,
            &mut self.diagnostics,
            &mut self.boot,
            &mut self.network,
            &mut self.packages,
            &mut self.users,
            &mut self.configs,
            &mut self.boot_repair,
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
