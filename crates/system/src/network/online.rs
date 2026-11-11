//! What the Network page asks outside the snapshot: NetworkManager's
//! connectivity over D-Bus. The public address is `toolbox-services`'s,
//! and what the page does outside a plan is in `actions`.

use std::time::Duration;

use toolbox_plan::UnitScope;
use toolbox_state::network::Connectivity;
use zbus::OwnedVariant;

use super::network_manager::{
    CHECK_CONNECTIVITY, CONNECTIVITY_PROPERTY, connectivity_from_value, connectivity_property_body,
};
use crate::bus::{CallError, call, call_with_timeout, connect};

/// NetworkManager's own check waits for its test page. The page shows
/// "unknown" rather than wait longer.
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

/// What NetworkManager says about the way to the internet, unknown
/// without it. With `recheck` it checks now, which makes NetworkManager
/// fetch its test page. Without, the state of its last check is read.
pub async fn connectivity(recheck: bool) -> Connectivity {
    let connection = match connect(UnitScope::System).await {
        Ok(connection) => connection,
        Err(error) => {
            tracing::debug!("no connectivity state: {error}");
            return Connectivity::Unknown;
        }
    };

    if recheck {
        let check =
            call_with_timeout::<_, u32>(&connection, &CHECK_CONNECTIVITY, &(), CHECK_TIMEOUT);

        match check.await {
            Ok(code) => return Connectivity::from_code(code),
            Err(CallError::Unavailable) => return Connectivity::Unknown,
            // A session that may not ask for a check, or a check that took
            // too long, still leaves the state of the last one to read.
            Err(error) => tracing::debug!("NetworkManager did not check: {error}"),
        }
    }

    let body = connectivity_property_body();

    match call::<_, OwnedVariant>(&connection, &CONNECTIVITY_PROPERTY, &body).await {
        Ok(value) => connectivity_from_value(&value),
        Err(error) => {
            tracing::debug!("no connectivity state: {error}");
            Connectivity::Unknown
        }
    }
}
