//! `org.freedesktop.Application` on the session bus.
//!
//! The interface follows the Desktop Entry specification's D-Bus
//! activation section. The app is not D-Bus activatable: the interface
//! only lets a later start reach the first one.
//!
//! The first start claims the name before the window opens. A later
//! start finds it taken, calls `Activate` or `ActivateAction` with its
//! tokens and exits. Anything that fails or hangs runs the app alone, so
//! a broken bus never keeps the window from opening.

pub mod queue;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use zbus::fdo::{RequestNameFlags, RequestNameReply};
use zbus::{OwnedVariant, Variant};

use crate::activation::{Activation, Call};

const BUS_NAME: &str = "org.garudalinux.toolbox";
/// The specification's path for the name: dots become slashes.
const OBJECT_PATH: &str = "/org/garudalinux/toolbox";
const INTERFACE: &str = "org.freedesktop.Application";

/// Each of the claim and the hand-over gets this long. Both run before
/// the window exists, so a hang looks like a click that does nothing.
pub const BUS_LIMIT: Duration = Duration::from_secs(3);

/// Where the first start takes the wishes of later ones. Called on the
/// bus's threads.
pub trait Arrivals: Send + Sync + 'static {
    fn arrive(&self, activation: Activation);
}

pub enum Start<H> {
    /// This is the first start. Later ones reach [`Arrivals`] while `H`
    /// lives.
    First(H),
    Handed,
    /// No bus, or nobody answered: run without the check.
    Alone,
}

/// The two talks with the bus, so the decision is tested without one.
pub trait Bus {
    type Held;
    type Error: Display;

    /// `None` when another start owns the name.
    fn claim(&self) -> impl Future<Output = Result<Option<Self::Held>, Self::Error>> + Send;
    fn hand_over(
        &self,
        activation: &Activation,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// The user's session bus. A clone of the arrivals lives in the claimed
/// name's object server for as long as the app holds the name.
pub struct SessionBus {
    arrivals: Arc<dyn Arrivals>,
    desk: Option<Arc<dyn queue::Desk>>,
}

pub struct Held {
    _connection: zbus::Connection,
}

struct Application {
    arrivals: Arc<dyn Arrivals>,
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl Application {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn activate(&self, platform_data: HashMap<String, OwnedVariant>) {
        self.arrivals
            .arrive(Activation::default().with_platform_data(strings(&platform_data)));
    }

    #[expect(
        clippy::needless_pass_by_value,
        clippy::unused_self,
        reason = "the specification's method, which zbus hands its owned arguments"
    )]
    fn open(
        &self,
        uris: Vec<String>,
        platform_data: HashMap<String, OwnedVariant>,
    ) -> zbus::fdo::Result<()> {
        tracing::debug!(
            "refused to open {} links with {} platform data entries",
            uris.len(),
            platform_data.len()
        );

        Err(zbus::fdo::Error::NotSupported(
            "Garuda Toolbox opens no files".to_owned(),
        ))
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn activate_action(
        &self,
        action_name: &str,
        parameter: Vec<OwnedVariant>,
        platform_data: HashMap<String, OwnedVariant>,
    ) -> zbus::fdo::Result<()> {
        let Some(texts) = parameter.iter().map(text).collect::<Option<Vec<_>>>() else {
            return Err(zbus::fdo::Error::InvalidArgs(
                "the parameters must be strings".to_owned(),
            ));
        };

        let activation = Activation::from_action(action_name, &texts)
            .map_err(|refusal| zbus::fdo::Error::InvalidArgs(refusal.as_str().to_owned()))?
            .with_platform_data(strings(&platform_data));

        self.arrivals.arrive(activation);
        Ok(())
    }
}

impl SessionBus {
    #[must_use = "the bus does nothing until claimed"]
    pub fn new(arrivals: Arc<dyn Arrivals>) -> Self {
        Self {
            arrivals,
            desk: None,
        }
    }

    #[must_use = "the bus does nothing until claimed"]
    pub fn with_desk(mut self, desk: Arc<dyn queue::Desk>) -> Self {
        self.desk = Some(desk);
        self
    }
}

impl Bus for SessionBus {
    type Held = Held;
    type Error = zbus::Error;

    async fn claim(&self) -> zbus::Result<Option<Held>> {
        let connection = zbus::Connection::session().await?;
        let application = Application {
            arrivals: Arc::clone(&self.arrivals),
        };

        connection
            .object_server()
            .at(OBJECT_PATH, application)
            .await?;

        if let Some(desk) = &self.desk {
            let queue = queue::Queue {
                desk: Arc::clone(desk),
                limit: queue::ANSWER_LIMIT,
            };

            connection.object_server().at(OBJECT_PATH, queue).await?;
        }

        let reply = connection
            .request_name_with_flags(BUS_NAME, RequestNameFlags::DO_NOT_QUEUE)
            .await;

        Ok(owned(reply)?.then_some(Held {
            _connection: connection,
        }))
    }

    async fn hand_over(&self, activation: &Activation) -> zbus::Result<()> {
        let connection = zbus::Connection::session().await?;
        let platform_data: HashMap<&str, Variant<'_>> = activation
            .platform_data()
            .into_iter()
            .map(|(key, token)| (key, Variant::from(token)))
            .collect();

        match activation.call() {
            Call::Activate => {
                connection
                    .call_method(
                        Some(BUS_NAME),
                        OBJECT_PATH,
                        Some(INTERFACE),
                        "Activate",
                        &(platform_data,),
                    )
                    .await?;
            }
            Call::ActivateAction { name, parameters } => {
                let parameters: Vec<Variant<'_>> =
                    parameters.into_iter().map(Variant::from).collect();

                connection
                    .call_method(
                        Some(BUS_NAME),
                        OBJECT_PATH,
                        Some(INTERFACE),
                        "ActivateAction",
                        &(name, parameters, platform_data),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

pub async fn settle<B>(bus: &B, activation: &Activation, limit: Duration) -> Start<B::Held>
where
    B: Bus + Sync,
    B::Held: Send,
{
    let seconds = limit.as_secs();

    match tokio::time::timeout(limit, bus.claim()).await {
        Ok(Ok(Some(held))) => Start::First(held),
        Ok(Ok(None)) => match tokio::time::timeout(limit, bus.hand_over(activation)).await {
            Ok(Ok(())) => {
                tracing::info!("handed the start over to the running Garuda Toolbox");
                Start::Handed
            }
            Ok(Err(error)) => {
                tracing::warn!("cannot reach the running Garuda Toolbox, starting alone: {error}");
                Start::Alone
            }
            Err(_elapsed) => {
                tracing::warn!(
                    "the running Garuda Toolbox did not answer in {seconds} s, starting alone"
                );
                Start::Alone
            }
        },
        Ok(Err(error)) => {
            tracing::warn!("no session bus, so a second start opens a second window: {error}");
            Start::Alone
        }
        Err(_elapsed) => {
            tracing::warn!("the session bus did not answer in {seconds} s, starting alone");
            Start::Alone
        }
    }
}

fn text(value: &OwnedVariant) -> Option<&str> {
    if let Variant::Str(text) = &**value {
        Some(text.as_str())
    } else {
        None
    }
}

fn strings(data: &HashMap<String, OwnedVariant>) -> impl Iterator<Item = (&str, Option<&str>)> {
    data.iter().map(|(key, value)| (key.as_str(), text(value)))
}

/// zbus answers a taken name with an error of its own rather than with `Exists`.
fn owned(reply: zbus::Result<RequestNameReply>) -> zbus::Result<bool> {
    match reply {
        Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner) => Ok(true),
        Ok(RequestNameReply::Exists | RequestNameReply::InQueue) | Err(zbus::Error::NameTaken) => {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}
