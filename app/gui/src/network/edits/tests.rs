use toolbox_core::features::network::Refusal;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::KernelModule;

use crate::command::{AppCommand, NetworkCommand};
use crate::network::Network;

#[test]
fn a_blacklist_click_before_the_drivers_are_read_is_refused() {
    let mut network = Network::default();
    let command = AppCommand::Network(NetworkCommand::Blacklist {
        module: KernelModule::new("r8169").unwrap(),
        on: true,
    });

    let handled = network.handle(&command, &ChangeSet::default(), &Snapshot::default());

    assert!(handled.edit.is_none());
    let refused = network.refusal().unwrap();
    assert_eq!(refused.refusal, Refusal::Unavailable);
    assert_eq!(refused.subject, "r8169");
}
