//! The Network page on the real system: the connection, the drivers of
//! the running kernel, the devices and the tests. A dry run asks no
//! address service, it answers a TEST-NET address. A host that is no
//! host leaves Run ping greyed.

use egui::Key;
use egui::accesskit::Role;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Network"),
        Step::wait("Internet"),
        Step::WaitGone("Checking…"),
        Step::WaitGone("Reading…"),
        Step::button("Show Public Address"),
        Step::WaitContaining("203.0.113.7"),
        Step::Shot("network"),
        Step::WaitContaining("Good (57 %)"),
        Step::Shot("network-cards"),
        Step::WaitContaining("1000 Mb/s"),
        Step::radio("Drivers"),
        Step::WaitEither("Search drivers", "No network drivers found"),
        Step::Shot("network-drivers"),
        Step::radio("Hardware"),
        Step::WaitGone("Reading the devices…"),
        Step::Shot("network-hardware"),
        Step::radio("Tools"),
        Step::Click {
            role: Some(Role::TextInput),
            label: "Host",
        },
        Step::shortcut(Key::A),
        Step::Text("not a host!"),
        Step::WaitContaining("Enter a host name"),
        Step::disabled_button("Run Ping"),
        Step::Shot("network-tools"),
    ]
}
