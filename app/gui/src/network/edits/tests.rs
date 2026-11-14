use toolbox_core::features::network::Refusal;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{KernelModule, UnitScope};

use crate::command::{AppCommand, NetworkCommand, ServiceCommand};
use crate::network::{Address, Network};
use crate::page::PageId;
use crate::page_service::PageService as _;
use crate::refused::Refused;

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

#[test]
fn dismissing_takes_only_the_network_refusal_away() {
    let mut network = Network::default();
    network.set_refusal(Some(Refused::new(Refusal::Unavailable, "r8169".to_owned())));

    let other = network.receive(
        &AppCommand::DismissRefusal(PageId::Packages),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(!other.taken);
    assert!(network.refusal().is_some());

    let handled = network.receive(
        &AppCommand::DismissRefusal(PageId::Network),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert!(network.refusal().is_none());
}

/// The page lists the system's units too, which App reads in.
#[test]
fn refresh_asks_for_the_system_and_its_units() {
    let mut network = Network::default();

    let handled = network.handle(
        &AppCommand::Network(NetworkCommand::Refresh),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(handled.taken);
    assert_eq!(
        handled.answers,
        [
            AppCommand::ReadSystem,
            AppCommand::Service(ServiceCommand::Refresh(UnitScope::System)),
        ]
    );
}

#[test]
fn the_address_clicks_are_the_services() {
    let mut network = Network::default();

    for command in [NetworkCommand::LookupAddress, NetworkCommand::ForgetAddress] {
        let handled = network.handle(
            &AppCommand::Network(command),
            &ChangeSet::default(),
            &Snapshot::default(),
        );

        assert!(handled.taken);
        assert_eq!(handled.answers, []);
    }

    assert_eq!(network.address(), &Address::NotAsked);
}

/// A run at once needs App's Console.
#[test]
fn a_driver_load_is_left_to_app() {
    let mut network = Network::default();

    let handled = network.handle(
        &AppCommand::Network(NetworkCommand::LoadDriver {
            module: KernelModule::new("r8169").unwrap(),
            load: true,
        }),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(!handled.taken);
}
