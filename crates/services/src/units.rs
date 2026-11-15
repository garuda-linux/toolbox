//! Listing the units of a scope from systemd1 over D-Bus: system units
//! from the system bus, the user's own from the session bus.

use std::sync::Arc;

use futures_lite::future::Boxed;
use toolbox_core::features::services::UnitInfo;
use toolbox_plan::UnitScope;
use toolbox_system::systemd::{LIST_UNIT_FILES, LIST_UNITS, ListUnitsRow, UnitFileRow, unit_infos};
use toolbox_system::{CallError, call, forget_shared, shared};

pub type UnitList = Result<Arc<Vec<UnitInfo>>, CallError>;

/// Lists the units of a scope. Tests answer from memory.
pub trait UnitLister: Send + Sync {
    fn list(&self, scope: UnitScope) -> Boxed<UnitList>;
}

#[derive(Clone, Copy, Debug)]
pub struct Systemd;

impl UnitLister for Systemd {
    fn list(&self, scope: UnitScope) -> Boxed<UnitList> {
        Box::pin(async move {
            let listed = list(scope).await;
            if listed.is_err() {
                forget_shared(scope);
            }

            listed
        })
    }
}

/// Over the connection the watcher of the scope shares. A failure drops
/// it, as the bus may have gone.
async fn list(scope: UnitScope) -> UnitList {
    let connection = shared(scope).await?;
    let units: Vec<ListUnitsRow> = call(&connection, &LIST_UNITS, &()).await?;
    let files: Vec<UnitFileRow> = call(&connection, &LIST_UNIT_FILES, &()).await?;

    Ok(Arc::new(unit_infos(units, files)))
}
