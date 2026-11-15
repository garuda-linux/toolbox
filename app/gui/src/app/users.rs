//! What the app does with a click on the Users and groups page beyond its
//! service, which queues the changes. A new account and a new password run
//! at once, with the Console closed: the page shows how they went.

use toolbox_core::features::users::Refusal;
use toolbox_core::features::users::now::{plan_add_user, plan_set_password};
use toolbox_core::features::users::run::RunKind;

use super::App;
use crate::command::UsersCommand;
use crate::page_service::Owner;
use crate::refused::Refused;

impl App {
    pub(super) fn run_users(&mut self, command: UsersCommand) {
        match command {
            UsersCommand::AddUser { user, password } => {
                let name = user.parts().name.clone();
                let refusal = if self.run_unseen(plan_add_user(*user, password), Owner::Users) {
                    self.services.users.started(RunKind::NewAccount, name);
                    None
                } else {
                    Some(Refused::new(Refusal::Busy, name.to_string()))
                };

                self.services.users.set_refusal(refusal);
            }
            UsersCommand::SetPassword { user, password } => {
                let snapshot = self.current_snapshot();
                let result = plan_set_password(&user, password, &snapshot).and_then(|plan| {
                    if !self.run_unseen(plan, Owner::Users) {
                        return Err(Refusal::Busy);
                    }

                    self.services.users.started(RunKind::Password, user.clone());
                    Ok(())
                });

                let refusal = result
                    .err()
                    .map(|refusal| Refused::new(refusal, user.to_string()));

                self.services.users.set_refusal(refusal);
            }
            UsersCommand::ShowConsole => self.state.console_open = true,
            // The service takes these, see `hand_to_services`.
            UsersCommand::Keep(_)
            | UsersCommand::Edit(_)
            | UsersCommand::Refresh
            | UsersCommand::DismissRun => {}
        }
    }
}
