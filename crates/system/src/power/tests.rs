use zbus::zvariant::Type as _;

use super::{InhibitBody, RebootBody, inhibit_body, reboot_body};

#[test]
fn the_body_matches_the_reboot_signature() {
    assert_eq!(RebootBody::SIGNATURE.to_string_no_parens(), "b");
}

#[test]
fn a_reboot_lets_polkit_ask_the_user() {
    assert_eq!(reboot_body(), (true,));
}

#[test]
fn the_inhibit_body_matches_its_signature() {
    assert_eq!(InhibitBody::SIGNATURE.to_string_no_parens(), "ssss");
}

/// `block` holds a shutdown back until the lock goes, `delay` would only
/// slow it down.
#[test]
fn an_apply_blocks_shutdown_and_sleep() {
    let (what, who, _why, mode) = inhibit_body("Installing updates");

    assert_eq!(what, "shutdown:sleep");
    assert_eq!(who, "Garuda Toolbox");
    assert_eq!(mode, "block");
}
