use egui_kittest::kittest::Queryable as _;

use super::{Refusals, module, page, sent, tab};
use crate::command::{Command, NetworkCommand};

#[test]
fn unloading_a_driver_asks_first_and_keeps_it_on_no() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Drivers");

    harness.get_by_label("Unload…").click();
    harness.run();

    assert!(
        harness
            .query_all_by_label("Unload e1000e?")
            .next()
            .is_some()
    );
    assert_eq!(sent(&harness), []);

    harness.get_by_label("Keep it loaded").click();
    harness.run();
    assert_eq!(sent(&harness), []);

    harness.get_by_label("Unload…").click();
    harness.run();
    harness.get_by_label("Unload").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::LoadDriver {
            module: module("e1000e"),
            load: false,
        })]
    );
}
