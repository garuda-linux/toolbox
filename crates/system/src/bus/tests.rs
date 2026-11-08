use std::io;
use std::sync::Arc;

use zbus::fdo;

use super::{CallError, classify};

#[test]
fn a_missing_service_is_unavailable() {
    let error = zbus::Error::FDO(Box::new(fdo::Error::ServiceUnknown(
        "org.freedesktop.Notifications".to_owned(),
    )));

    assert_eq!(classify(&error), CallError::Unavailable);
}

#[test]
fn a_missing_bus_is_unavailable() {
    let no_address = zbus::Error::Address("no session bus".to_owned());
    let no_socket = zbus::Error::InputOutput(Arc::new(io::Error::from(io::ErrorKind::NotFound)));

    assert_eq!(classify(&no_address), CallError::Unavailable);
    assert_eq!(classify(&no_socket), CallError::Unavailable);
}

#[test]
fn a_refused_authorization_is_denied() {
    let error = zbus::Error::FDO(Box::new(fdo::Error::AccessDenied(
        "Interactive authentication required.".to_owned(),
    )));

    assert_eq!(classify(&error), CallError::Denied);
}

#[test]
fn other_errors_keep_their_name_and_text() {
    let error = zbus::Error::FDO(Box::new(fdo::Error::Failed("busy".to_owned())));

    assert_eq!(
        classify(&error),
        CallError::Failed("org.freedesktop.DBus.Error.Failed: busy".to_owned())
    );
}

#[test]
fn errors_without_a_name_keep_their_text() {
    let error = zbus::Error::Failure("handshake".to_owned());

    assert_eq!(classify(&error), CallError::Failed(error.to_string()));
}
