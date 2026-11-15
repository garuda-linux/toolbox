use std::future::Ready;
use std::sync::atomic::{AtomicU32, Ordering};

use toolbox_plan::UnitScope;

use super::Buses;
use crate::bus::CallError;

/// Counts the connections made, each one named by its number.
#[derive(Default)]
struct Bus(AtomicU32);

impl Bus {
    fn connect(&self) -> Ready<Result<u32, CallError>> {
        std::future::ready(Ok(self.0.fetch_add(1, Ordering::SeqCst)))
    }
}

#[tokio::test]
async fn each_scope_connects_once_and_again_after_a_failure() {
    let buses = Buses::new();
    let bus = Bus::default();

    let first = buses.get(UnitScope::System, || bus.connect()).await;
    let again = buses.get(UnitScope::System, || bus.connect()).await;
    let user = buses.get(UnitScope::User, || bus.connect()).await;
    buses.forget(UnitScope::System);
    let after = buses.get(UnitScope::System, || bus.connect()).await;

    assert_eq!(first, Ok(0));
    assert_eq!(again, Ok(0));
    assert_eq!(user, Ok(1));
    assert_eq!(after, Ok(2));
}

#[tokio::test]
async fn a_failed_connect_keeps_nothing() {
    let buses = Buses::new();
    let bus = Bus::default();

    let failed = buses
        .get(UnitScope::User, || {
            std::future::ready(Err(CallError::Unavailable))
        })
        .await;

    let next = buses.get(UnitScope::User, || bus.connect()).await;

    assert_eq!(failed, Err(CallError::Unavailable));
    assert_eq!(next, Ok(0));
}
