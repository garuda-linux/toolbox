//! The reads on the system bus: `GetAll` of NetworkManager's objects and
//! the `PropertiesChanged` signals below its root.

use std::collections::HashMap;

use serde_json::Value;
use toolbox_plan::UnitScope;
use toolbox_state::network::InterfaceCard;
use zbus::{Connection, MatchRule, MessageStream, OwnedVariant};

use super::json::properties_json;
use super::{DESTINATION, PropertyReads, ROOT_PATH, cards};
use crate::bus::{CALL_TIMEOUT, CallError, bounded, classify, forget_shared, shared};

const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// The system connection the units share, see [`shared`].
struct SystemProperties(pub Connection);

impl PropertyReads for SystemProperties {
    async fn all(&self, path: &str, interface: &str) -> Result<Value, CallError> {
        let reply = bounded(
            CALL_TIMEOUT,
            self.0.call_method(
                Some(DESTINATION),
                path,
                Some(PROPERTIES),
                "GetAll",
                &interface,
            ),
        )
        .await?;

        let properties: HashMap<String, OwnedVariant> = reply
            .body()
            .deserialize()
            .map_err(|error| classify(&error))?;

        Ok(properties_json(&properties))
    }
}

/// The cards of the system's NetworkManager.
///
/// # Errors
///
/// [`CallError::Unavailable`] without a bus or NetworkManager on it.
pub async fn system_cards() -> Result<Vec<InterfaceCard>, CallError> {
    let connection = shared(UnitScope::System).await?;
    let read = cards(&SystemProperties(connection)).await;

    if read.is_err() {
        forget_shared(UnitScope::System);
    }

    read
}

/// The `PropertiesChanged` signals of NetworkManager and its objects,
/// matched by the bus so other signals never wake the caller.
///
/// # Errors
///
/// The failure to add the match rule, classified.
pub async fn property_changes(connection: &Connection) -> Result<MessageStream, CallError> {
    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(DESTINATION)
        .interface(PROPERTIES)
        .member("PropertiesChanged")
        .path_namespace(ROOT_PATH)
        .build()
        .map_err(|error| classify(&error))?;

    MessageStream::for_match_rule(rule, connection, None)
        .await
        .map_err(|error| classify(&error))
}
