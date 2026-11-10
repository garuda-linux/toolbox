//! Loading and unloading at once, as the drivers list's button offers it.

use super::drivers::{facts, module};
use crate::Toggle;
use crate::features::network::{SwitchRefusal, plan_load, plan_switch, plan_unload};

#[test]
fn a_listed_driver_loads_when_it_is_not_loaded_and_unloads_when_it_is() {
    let facts = facts(Vec::new());

    assert_eq!(
        plan_switch(&module("mt76-usb"), Toggle::On, &facts),
        Ok(plan_load(&module("mt76-usb")))
    );
    assert_eq!(
        plan_switch(&module("r8169"), Toggle::Off, &facts),
        Ok(plan_unload(&module("r8169")))
    );
}

#[test]
fn a_driver_already_in_that_state_is_nothing_to_do() {
    let facts = facts(Vec::new());

    assert_eq!(
        plan_switch(&module("r8169"), Toggle::On, &facts),
        Err(SwitchRefusal::AlreadyLoaded)
    );
    assert_eq!(
        plan_switch(&module("mt76-usb"), Toggle::Off, &facts),
        Err(SwitchRefusal::NotLoaded)
    );
}

/// The list holds network drivers only, so the button never names others.
#[test]
fn a_module_the_list_lacks_is_refused() {
    assert_eq!(
        plan_switch(&module("snd_hda_intel"), Toggle::On, &facts(Vec::new())),
        Err(SwitchRefusal::NotListed)
    );
}
