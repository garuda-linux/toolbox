use toolbox_core::Toggle;
use toolbox_core::features::tweaks::{self, Tweak};
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

        let tweak = match command {
            TweakCommand::Shell(shell) => {
                let Some(user) = user else {
                    return;
                };

                Tweak::Shell(user, shell)
            }
            TweakCommand::Dns(provider) => Tweak::Dns(provider),
            TweakCommand::Wifi(backend) => Tweak::Wifi(backend),
            TweakCommand::Hblock(on) => Tweak::Hblock(Toggle::from_on(on)),
            TweakCommand::Flag(flag, on) => Tweak::Flag(flag, Toggle::from_on(on)),
        };

        self.queue.edit(&snapshot, |changes, snapshot| {
            tweaks::queue(changes, tweak, snapshot);
        });
    }
}
