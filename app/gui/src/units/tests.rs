use std::sync::Arc;
use std::time::Duration;

use futures_lite::future::Boxed;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_plan::{UnitName, UnitScope};
use toolbox_system::CallError;

use super::{UnitList, UnitLister, UnitLists, UnitsFailure, UnitsHandle, spawn};
use crate::testing::PATIENCE;

struct Recorded {
    asked: crossbeam_channel::Sender<UnitScope>,
}

impl UnitLister for Recorded {
    fn list(&self, scope: UnitScope) -> Boxed<UnitList> {
        self.asked.send(scope).unwrap();
        Box::pin(async move {
            match scope {
                UnitScope::System => Ok(Arc::new(vec![unit("sshd.service")])),
                UnitScope::User => Err(CallError::Unavailable),
            }
        })
    }
}

fn unit(name: &str) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: toolbox_text::plain(""),
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        file_state: FileState::Enabled,
    }
}

async fn units_where(handle: &mut UnitsHandle, wanted: impl Fn(&UnitLists) -> bool) -> UnitLists {
    tokio::time::timeout(PATIENCE, async {
        loop {
            if let Some(units) = handle.changed()
                && wanted(&units)
            {
                return units;
            }

            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}

/// A settle longer than the patience shows that the first look waits
/// for none.
#[tokio::test]
async fn nothing_is_listed_before_a_page_opens_and_then_both_scopes_at_once() {
    let (asked, answered) = crossbeam_channel::unbounded();
    let mut handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        Arc::new(Recorded { asked }),
        PATIENCE * 2_u32,
    );

    handle.refresh(UnitScope::System);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(answered.try_iter().count(), 0, "no page opened");

    handle.open();
    handle.open();
    let units = units_where(&mut handle, |units| {
        units.system.get().is_some() && units.user.error().is_some()
    })
    .await;

    let system = units.get(UnitScope::System).get().unwrap();
    assert_eq!(
        system.first().map(|unit| unit.name.as_str()),
        Some("sshd.service")
    );
    assert_eq!(units.user.error(), Some(&UnitsFailure::Unavailable));
    assert_eq!(
        answered.try_iter().count(),
        2,
        "a second open lists nothing"
    );
}

#[tokio::test]
async fn signals_while_settling_list_each_scope_once() {
    let (asked, answered) = crossbeam_channel::unbounded();
    let mut handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        Arc::new(Recorded { asked }),
        Duration::from_millis(50),
    );

    handle.open();
    units_where(&mut handle, |units| {
        units.system.get().is_some() && units.user.error().is_some()
    })
    .await;

    let signals = handle.requests();
    for _ in 0_u8..5 {
        signals.refresh(UnitScope::System);
    }

    tokio::time::sleep(Duration::from_millis(200)).await;

    let listed: Vec<UnitScope> = answered.try_iter().collect();
    assert_eq!(
        listed,
        [UnitScope::System, UnitScope::User, UnitScope::System]
    );
}

#[test]
fn a_failed_list_keeps_the_last_units_beside_its_failure() {
    let mut units = UnitLists::default();
    units.finish(UnitScope::System, Ok(Arc::new(vec![unit("sshd.service")])));

    units.finish(
        UnitScope::System,
        Err(CallError::Failed("timed out".to_owned())),
    );

    assert_eq!(units.system.error(), Some(&UnitsFailure::Failed));
    assert_eq!(units.system.get().map(Vec::len), Some(1));
    assert_eq!(units.user, toolbox_gui_kit::Loadable::NotLoaded);
}

#[tokio::test]
async fn the_units_await_only_lists_an_open_page_asked_for() {
    let (asked, _answered) = crossbeam_channel::unbounded();
    let handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        Arc::new(Recorded { asked }),
        Duration::ZERO,
    );

    let mut units = super::Units::new(Some(handle));
    assert!(!units.awaits(), "no page opened");

    units.open();
    assert!(units.awaits());

    tokio::time::timeout(PATIENCE, async {
        while units.awaits() {
            tokio::time::sleep(Duration::from_millis(5)).await;
            units.take();
        }
    })
    .await
    .unwrap();

    assert!(
        units.get(UnitScope::User).error().is_some(),
        "a failure ends the wait"
    );
}
