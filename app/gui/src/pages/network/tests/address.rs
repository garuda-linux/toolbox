use std::net::{IpAddr, Ipv4Addr};

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{network_with, page_showing, sent};
use crate::command::{AppCommand, Command};
use crate::network::Address;
use crate::testing::rect_of;

#[test]
fn a_known_address_sits_at_the_right_like_the_other_values() {
    let known = Address::Known(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));
    let harness = page_showing(network_with(known));

    let title = rect_of(&harness, Role::Label, "Public address");
    let value = rect_of(&harness, Role::Label, "203.0.113.7");
    assert!(
        value.left() > title.right() && (value.center().y - title.center().y).abs() < 2.0,
        "the address at {value:?} is not beside its title at {title:?}"
    );
}

#[test]
fn the_copy_icon_copies_the_address() {
    let known = Address::Known(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));
    let mut harness = page_showing(network_with(known));

    harness
        .get_by_role_and_label(Role::Button, "Copy public address")
        .click();
    harness.run();

    assert!(
        sent(&harness).iter().any(|command| matches!(
            command,
            Command::App(AppCommand::Toast { kind: _, text: _ })
        )),
        "no copied note"
    );
}
