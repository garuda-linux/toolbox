//! What the app does with a click on the Users and groups page beyond its
//! service, which queues the changes. A new account and a new password run
//! at once, with the Console closed: the page shows how they went.

use toolbox_core::features::users::now::{plan_add_user, plan_set_password};
use toolbox_core::features::users::run::RunKind;

use super::App;
use crate::command::UsersCommand;
use crate::refused::Refused;

impl App {
    pub(super) fn run_users(&mut self, command: UsersCommand) {
        match command {
            UsersCommand::Refresh => {
                self.read_system();
                self.services.users.refresh();
            }
            UsersCommand::AddUser { user, password } => {
                let name = user.parts().name.clone();
                if self.run_unseen(plan_add_user(*user, password)) {
                    self.services.users.started(RunKind::NewAccount, name);
                }
            }
            UsersCommand::SetPassword { user, password } => {
                let snapshot = self.current_snapshot();
                let result = plan_set_password(&user, password, &snapshot).map(|plan| {
                    if self.run_unseen(plan) {
                        self.services.users.started(RunKind::Password, user.clone());
                    }
                });

                let refusal = result.err().map(|refusal| Refused {
                    refusal,
                    subject: user.to_string(),
                });

                self.services.users.set_refusal(refusal);
            }
            UsersCommand::DismissRun => self.services.users.dismiss_run(),
            UsersCommand::ShowConsole => self.state.console_open = true,
            UsersCommand::Keep(_) | UsersCommand::DismissRefusal | UsersCommand::Edit(_) => {}
        }
    }
}
