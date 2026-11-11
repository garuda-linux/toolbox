//! Each service's hooks, in the shape [`PageService`] asks for. The
//! services keep their own methods, which their tests call.

use std::path::Path;

use toolbox_core::Snapshot;
use toolbox_core::domains::BootLoader;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::grub;
use toolbox_core::features::bootrepair::page::Booted;
use toolbox_services::diagnostics::page::Changed;
use toolbox_system::Host;

use super::{Answer, PageService};
use crate::boot::{Boot, FileKind};
use crate::bootrepair::BootRepair;
use crate::command::AppCommand;
use crate::configs::Configs;
use crate::diagnostics::Diagnostics;
use crate::maintenance::Maintenance;
use crate::network::Network;
use crate::packages::Packages;
use crate::page::PageId;
use crate::users::Users;

impl PageService for Maintenance {
    /// A ranked list comes back as the plan that installs it.
    fn take(&mut self, _page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        answers(Self::take(self).map(Answer::RunPlan))
    }

    fn plan_started(&mut self) -> Vec<AppCommand> {
        Self::plan_started(self);
        Vec::new()
    }

    fn plan_refused(&mut self) -> Vec<AppCommand> {
        Self::plan_refused(self);
        Vec::new()
    }

    /// The last check of the programs may predate the end, so the disk is
    /// asked.
    fn plan_finished(&mut self, succeeded: bool) -> Vec<AppCommand> {
        let launcher = self.launcher();
        Self::plan_finished(self, succeeded, Path::exists, launcher.as_deref());
        Vec::new()
    }
}

impl PageService for Diagnostics {
    /// A finished report or upload, for the clipboard.
    fn take(&mut self, _page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        let changes = Self::take(self);
        let copies = changes.into_iter().filter_map(|changed| match changed {
            Changed::Report { ready: true } => Some(Answer::Copy {
                text: self.page().report.clone(),
                toast: "diagnostics-report-copied",
            }),
            Changed::Upload { done: true } => self.page().share.paste().map(|paste| Answer::Copy {
                text: paste.link.clone(),
                toast: "share-link-copied",
            }),
            Changed::Status
            | Changed::Report { ready: false }
            | Changed::Upload { done: false } => None,
        });

        answers(copies)
    }
}

impl PageService for Boot {
    /// The systemd-boot settings come with the menu, and a wish the file
    /// now meets drops out of the queue.
    fn take(&mut self, page: PageId, snapshot: &Snapshot) -> Vec<AppCommand> {
        if page == PageId::Boot {
            let sdboot = snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot);
            self.open(sdboot || grub(snapshot).is_ok());
        }

        let before = self.menu_identity();
        let picked = Self::take(self);
        let menu = (self.menu_identity() != before).then_some(Answer::Recombine);
        let edit = picked.map(|picked| {
            Answer::EditBoot(match picked.kind {
                FileKind::Background => Edit::Background(picked.path),
                FileKind::Theme => Edit::Theme(picked.path),
            })
        });

        answers(menu.into_iter().chain(edit))
    }

    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        Self::plan_finished(self);
        Vec::new()
    }
}

impl PageService for Network {
    fn take(&mut self, page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        if page == PageId::Network {
            self.open();
        }

        Self::take(self);
        Vec::new()
    }

    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        Self::plan_finished(self);
        Vec::new()
    }
}

impl PageService for Packages {
    fn take(&mut self, page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        if page == PageId::Packages {
            self.open();
        }

        Self::take(self);
        answers(self.aur_landed().then_some(Answer::Recombine))
    }

    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        Self::plan_finished(self);
        Vec::new()
    }
}

impl PageService for Users {
    /// New lock flags join the snapshot the pages share.
    fn take(&mut self, page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        if page == PageId::Users {
            self.open();
        }

        answers(Self::take(self).then_some(Answer::Recombine))
    }

    fn plan_started(&mut self) -> Vec<AppCommand> {
        Self::plan_started(self);
        Vec::new()
    }

    fn plan_refused(&mut self) -> Vec<AppCommand> {
        Self::plan_refused(self);
        Vec::new()
    }

    fn plan_finished(&mut self, succeeded: bool) -> Vec<AppCommand> {
        Self::plan_finished(self, succeeded);
        Vec::new()
    }
}

impl PageService for Configs {
    /// The page looks for its files once it shows, and takes what its
    /// work brought. An edit that needs the user brings the page back.
    fn take(&mut self, page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        if page == PageId::Configs {
            self.open();
        }

        let taken = Self::take(self);
        let review = taken.review.then_some(Answer::CloseConsole);
        answers(taken.acts.into_iter().map(Answer::Configs).chain(review))
    }

    fn plan_started(&mut self) -> Vec<AppCommand> {
        configs_act(Self::plan_started(self))
    }

    fn plan_refused(&mut self) -> Vec<AppCommand> {
        configs_act(Self::plan_refused(self))
    }

    fn plan_finished(&mut self, succeeded: bool) -> Vec<AppCommand> {
        configs_act(Self::plan_finished(self, succeeded))
    }
}

impl PageService for BootRepair {
    /// The live ISO may show up after the start, so the offer is worked
    /// out again on every frame.
    fn take(&mut self, page: PageId, snapshot: &Snapshot) -> Vec<AppCommand> {
        let live = snapshot.part::<Host>().is_some_and(|host| host.live_iso);
        self.see_live(Booted::of_live(live));
        let offered = self.offered();
        let leave = (page == PageId::BootRepair && !offered).then_some(Answer::LeaveBootRepair);
        if page == PageId::BootRepair && offered {
            self.open();
        }

        Self::take(self);
        answers(std::iter::once(Answer::OfferBootRepair(offered)).chain(leave))
    }

    fn plan_started(&mut self) -> Vec<AppCommand> {
        Self::plan_started(self);
        Vec::new()
    }

    fn plan_refused(&mut self) -> Vec<AppCommand> {
        Self::plan_refused(self);
        Vec::new()
    }

    fn plan_finished(&mut self, succeeded: bool) -> Vec<AppCommand> {
        Self::plan_finished(self, succeeded);
        Vec::new()
    }
}

fn answers(answers: impl IntoIterator<Item = Answer>) -> Vec<AppCommand> {
    answers.into_iter().map(AppCommand::Answer).collect()
}

fn configs_act(act: Option<crate::configs::Act>) -> Vec<AppCommand> {
    answers(act.map(Answer::Configs))
}
