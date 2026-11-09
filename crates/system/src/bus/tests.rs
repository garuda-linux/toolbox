use std::io;
use std::sync::Arc;
use std::time::Duration;

use zbus::fdo;

use super::client::bounded;
use super::{CALL_TIMEOUT, CallError, classify};

const LONGER: Duration = Duration::from_secs(15);

fn silent_bus() -> impl Future<Output = Result<(), zbus::Error>> {
    std::future::pending()
}

#[tokio::test(start_paused = true)]
async fn a_longer_timeout_outlasts_the_default_one() {
    let started = tokio::time::Instant::now();

    let answer = bounded(LONGER, silent_bus()).await;

    assert_eq!(answer, Err(CallError::Failed("timed out".to_owned())));
    assert_eq!(started.elapsed(), LONGER);
    assert!(started.elapsed() > CALL_TIMEOUT);
}

#[tokio::test(start_paused = true)]
async fn a_bus_that_answers_in_time_is_not_cut_off() {
    let late = async {
        tokio::time::sleep(CALL_TIMEOUT + Duration::from_secs(1)).await;
        Ok(7_u32)
    };

    assert_eq!(bounded(LONGER, late).await, Ok(7));
}

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
