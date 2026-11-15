use std::collections::BTreeMap;

use zbus::zvariant::Type as _;

use super::{Notification, NotifyBody, notify_body};

#[test]
fn the_body_matches_the_notify_signature() {
    assert_eq!(NotifyBody::SIGNATURE.to_string_no_parens(), "susssasa{sv}i");
}

#[test]
fn a_notification_names_the_app_and_has_no_actions() {
    let notification = Notification {
        summary: "Updates available".to_owned(),
        body: "12 packages".to_owned(),
    };

    let (app, replaces, icon, summary, body, actions, hints, timeout) = notify_body(&notification);

    assert_eq!(app, "Garuda Toolbox");
    assert_eq!(replaces, 0_u32);
    assert_eq!(icon, "garuda-toolbox");
    assert_eq!(summary, "Updates available");
    assert_eq!(body, "12 packages");
    assert_eq!(actions, Vec::<&str>::new());
    assert_eq!(hints, BTreeMap::new());
    assert_eq!(timeout, -1_i32);
}
