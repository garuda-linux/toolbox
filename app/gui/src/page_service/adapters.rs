//! Each service's hooks, in the shape [`PageService`] asks for. The
//! services keep their own methods, which their tests call.

mod runs;
#[cfg(test)]
mod tests;

use toolbox_core::domains::BootLoader;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::grub;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::UnitScope;

use super::{Answer, Handled, PageService};
use crate::boot::{Boot, FileKind};
use crate::command::AppCommand;
use crate::home::Home;
use crate::kernels::Kernels;
use crate::locales::Locales;
use crate::network::Network;
use crate::packages::Packages;
use crate::page::PageId;
use crate::toggle_pages::TogglePages;
use crate::units::Units;
use crate::users::Users;

impl PageService for Boot {
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        dismissed(page, PageId::Boot, || self.set_refusal(None))
    }

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
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        dismissed(page, PageId::Network, || self.set_refusal(None))
    }

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
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        dismissed(page, PageId::Packages, || self.set_refusal(None))
    }

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
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        dismissed(page, PageId::Users, || self.set_refusal(None))
    }

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

/// dkms only changes through the app's own plans, so it is not read again
/// when the window comes back.
impl PageService for Kernels {
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        dismissed(page, PageId::Kernels, || self.set_refusal(None))
    }

    fn take(&mut self, _page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        self.take_dkms();
        Vec::new()
    }

    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        self.read_dkms();
        Vec::new()
    }
}

impl PageService for Locales {
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        dismissed(page, PageId::Locales, || self.set_refusal(None))
    }
}

impl PageService for TogglePages {
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn dismiss_refusal(&mut self, page: PageId) -> bool {
        Self::dismiss_refusal(self, page)
    }
}

impl PageService for Units {
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    fn take(&mut self, _page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        Self::take(self);
        Vec::new()
    }

    /// systemd's signals report the change too, this covers a bus that
    /// dropped them.
    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        self.refresh(UnitScope::System);
        self.refresh(UnitScope::User);
        Vec::new()
    }
}

impl PageService for Home {
    fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        Self::handle(self, command, changes, snapshot)
    }

    /// The status and the programs are not read when the window comes
    /// back: the programs only change through the app's own plans, and
    /// the status checks reach the network, so they follow their period.
    fn take(&mut self, _page: PageId, _snapshot: &Snapshot) -> Vec<AppCommand> {
        Self::take(self);
        Vec::new()
    }

    fn plan_finished(&mut self, _succeeded: bool) -> Vec<AppCommand> {
        Self::plan_finished(self);
        Vec::new()
    }
}

fn answers(answers: impl IntoIterator<Item = Answer>) -> Vec<AppCommand> {
    answers.into_iter().map(AppCommand::Answer).collect()
}

/// Whether `page` is `own`, whose refusal `clear` drops then.
fn dismissed(page: PageId, own: PageId, clear: impl FnOnce()) -> bool {
    let own = page == own;
    if own {
        clear();
    }

    own
}
