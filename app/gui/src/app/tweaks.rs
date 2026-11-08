use toolbox_core::domains::{Dns, GarudaUpdateFlags, Hblock, LoginShells, WifiBackends};
use toolbox_core::{Single, Toggle};
use toolbox_system::Host;

use super::App;
use crate::command::TweakCommand;

impl App {
    /// A wish the system already fulfils is not queued, as on every page.
    pub(super) fn tweak(&mut self, command: TweakCommand) {
        let snapshot = self.current_snapshot();

        // The probe names the account. `USER` names the wrong one after
        // `sudo -u`, so it is no stand-in before that.
        let user = snapshot.part::<Host>().and_then(|host| host.user.clone());

        self.queue
            .edit(&snapshot, |changes, snapshot| match command {
                TweakCommand::Shell(shell) => {
                    if let Some(user) = user {
                        changes.want::<LoginShells>(user, shell, snapshot);
                    }
                }
                TweakCommand::Dns(provider) => changes.want::<Dns>(Single, provider, snapshot),
                TweakCommand::Wifi(backend) => {
                    changes.want::<WifiBackends>(Single, backend, snapshot);
                }
                TweakCommand::Hblock(on) => {
                    changes.want::<Hblock>(Single, Toggle::from_on(on), snapshot);
                }
                TweakCommand::Flag(flag, on) => {
                    changes.want::<GarudaUpdateFlags>(flag, Toggle::from_on(on), snapshot);
                }
            });
    }
}
