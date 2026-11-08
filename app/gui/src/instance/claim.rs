//! A start's claim on the session: the first start keeps the name and
//! takes later starts' wishes, a later one hands its wish over and ends.

use std::sync::Arc;

use tokio::runtime::Runtime;
use toolbox_commands::window::Opening;
use toolbox_instance::activation::Activation;
use toolbox_instance::bus::{BUS_LIMIT, Held, SessionBus, Start, settle};

use super::{Arrival, Arrivals};
use crate::command::{AppCommand, CommandBus};

pub struct Single {
    pub commands: CommandBus,
    pub arrivals: Option<Arc<Arrivals>>,
}

/// Claims the session's name, or hands `opening` to the app that has it.
/// `None`: the running app took the wish, so this start ends. The second
/// value keeps the name while the app runs.
pub fn claim(runtime: &Runtime, opening: Opening) -> Option<(Single, Option<Held>)> {
    let commands = CommandBus::new();
    let alone = |channel| Single {
        commands: channel,
        arrivals: None,
    };

    // The self-test's app must not hand its steps to the user's window.
    #[cfg(feature = "selftest")]
    if crate::selftest::requested(|key| std::env::var_os(key)).is_some() {
        return Some((alone(commands), None));
    }

    // A page action of this start runs once the window shows, as a later
    // start's would.
    if opening.action.is_some() {
        commands.sender.send(AppCommand::Activated(Arrival {
            page: opening.page,
            action: opening.action,
            raised: true,
        }));
    }

    let arrivals = Arc::new(Arrivals::new(commands.sender.clone()));
    let wanted = opening.activation();
    let wish = Activation {
        page: wanted.page,
        action: wanted.action,
        ..Activation::from_env(|key| std::env::var(key).ok())
    };

    let bus = SessionBus::new(Arc::<Arrivals>::clone(&arrivals))
        .with_desk(Arc::<Arrivals>::clone(&arrivals));

    match runtime.block_on(settle(&bus, &wish, BUS_LIMIT)) {
        Start::First(held) => Some((
            Single {
                commands,
                arrivals: Some(arrivals),
            },
            Some(held),
        )),
        Start::Handed => None,
        Start::Alone => Some((alone(commands), None)),
    }
}
