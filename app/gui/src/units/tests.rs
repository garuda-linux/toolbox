use std::sync::Arc;
use std::time::Duration;

use futures_lite::future::Boxed;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_plan::{UnitName, UnitScope};
use toolbox_system::CallError;

use super::{UnitList, UnitLister, Units, UnitsFailure, UnitsHandle, spawn};
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
        description: String::new(),
        active: ActiveState::Active,
        sub_state: "running".to_owned(),
        file_state: FileState::Enabled,
    }
}

async fn units_where(handle: &mut UnitsHandle, wanted: impl Fn(&Units) -> bool) -> Units {
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

#[tokio::test]
async fn both_scopes_are_listed_at_start() {
    let (asked, _answered) = crossbeam_channel::unbounded();
    let mut handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        Arc::new(Recorded { asked }),
        Duration::ZERO,
    );

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
}

#[tokio::test]
async fn requests_while_settling_list_each_scope_once() {
    let (asked, answered) = crossbeam_channel::unbounded();
    let mut handle = spawn(
        &tokio::runtime::Handle::current(),
        crate::waking::Wake::none(),
        Arc::new(Recorded { asked }),
        Duration::from_millis(50),
    );

    for _ in 0_u8..5 {
        handle.refresh(UnitScope::System);
    }

    units_where(&mut handle, |units| {
        units.system.get().is_some() && units.user.error().is_some()
    })
    .await;
    tokio::time::sleep(Duration::from_millis(150)).await;

    let listed: Vec<UnitScope> = answered.try_iter().collect();
    assert_eq!(listed, [UnitScope::System, UnitScope::User]);
}

#[test]
fn a_failed_list_keeps_the_last_units_beside_its_failure() {
    let mut units = Units::default();
    units.finish(UnitScope::System, Ok(Arc::new(vec![unit("sshd.service")])));

    units.finish(
        UnitScope::System,
        Err(CallError::Failed("timed out".to_owned())),
    );

    assert_eq!(units.system.error(), Some(&UnitsFailure::Failed));
    assert_eq!(units.system.get().map(Vec::len), Some(1));
    assert_eq!(units.user, toolbox_gui_kit::Loadable::NotLoaded);
}
