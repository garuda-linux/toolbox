//! A bounded D-Bus call and the bus for a unit scope. Each call opens its
//! own connection: calls are rare, and a held one would outlive a
//! restarted service.

use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use toolbox_plan::UnitScope;
use zbus::Connection;
use zbus::zvariant::{DynamicType, Type};

use super::{CallError, Method, classify};

pub const CALL_TIMEOUT: Duration = Duration::from_secs(10);

/// polkit's dialog waits for the user, but a call nobody answers must
/// still end.
pub const INTERACTIVE_TIMEOUT: Duration = Duration::from_secs(300);

/// # Errors
///
/// The failure to reach the bus, classified.
pub async fn connect(scope: UnitScope) -> Result<Connection, CallError> {
    let connection = match scope {
        UnitScope::System => Connection::system().await,
        UnitScope::User => Connection::session().await,
    };

    connection.map_err(|error| classify(&error))
}

/// `R` is `()` for a call that only has to succeed.
///
/// # Errors
///
/// [`CallError::Failed`] on a bus error or a reply that does not fit `R`,
/// after [`CALL_TIMEOUT`].
pub async fn call<B, R>(connection: &Connection, method: &Method, body: &B) -> Result<R, CallError>
where
    B: Serialize + DynamicType + Sync,
    R: DeserializeOwned + Type,
{
    call_with_timeout(connection, method, body, CALL_TIMEOUT).await
}

/// [`call`] for a method whose service takes longer than [`CALL_TIMEOUT`]
/// on purpose, such as a check that waits for its own test page.
///
/// # Errors
///
/// [`CallError::Failed`] on a bus error or a reply that does not fit `R`,
/// after `timeout`.
pub async fn call_with_timeout<B, R>(
    connection: &Connection,
    method: &Method,
    body: &B,
    timeout: Duration,
) -> Result<R, CallError>
where
    B: Serialize + DynamicType + Sync,
    R: DeserializeOwned + Type,
{
    let reply = bounded(
        timeout,
        connection.call_method(
            Some(method.destination),
            method.path,
            Some(method.interface),
            method.name,
            body,
        ),
    )
    .await?;

    reply
        .body()
        .deserialize::<R>()
        .map_err(|error| classify(&error))
}

pub(super) async fn bounded<T>(
    timeout: Duration,
    reply: impl Future<Output = Result<T, zbus::Error>>,
) -> Result<T, CallError> {
    tokio::time::timeout(timeout, reply)
        .await
        .map_err(|_elapsed| CallError::Failed("timed out".to_owned()))?
        .map_err(|error| classify(&error))
}
