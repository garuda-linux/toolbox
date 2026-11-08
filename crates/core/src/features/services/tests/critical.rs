use toolbox_plan::{UnitName, UnitOperation};

use super::super::{asks_first, critical};

fn unit(name: &str) -> UnitName {
    name.parse().unwrap()
}

#[test]
fn stopping_what_the_session_lives_on_asks_first() {
    for name in [
        "display-manager.service",
        "sddm.service",
        "gdm.service",
        "graphical.target",
        "NetworkManager.service",
        "polkit.service",
        "dbus.service",
        "dbus-broker.service",
        "systemd-logind.service",
    ] {
        let unit = unit(name);
        assert!(critical(&unit), "{name}");
        assert!(asks_first(UnitOperation::Stop, &unit), "{name}");
        assert!(asks_first(UnitOperation::Restart, &unit), "{name}");
        assert!(!asks_first(UnitOperation::Start, &unit), "{name}");
    }

    let bluetooth = unit("bluetooth.service");
    assert!(!critical(&bluetooth));
    assert!(!asks_first(UnitOperation::Stop, &bluetooth));
}
