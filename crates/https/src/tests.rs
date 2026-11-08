use std::io::Read as _;
use std::time::Duration;

use ureq::Body;
use ureq::http::Response;

use crate::{Clock as _, Https, RequestError, Rules, SystemClock};

const CAP: u64 = 16;

fn https() -> Https {
    Https::new(&Rules {
        timeout: Duration::from_secs(1),
        max_body_bytes: CAP,
        user_agent: Some("garuda-toolbox/test"),
    })
}

fn answer(bytes: usize) -> Response<Body> {
    Response::new(Body::builder().data(vec![b'a'; bytes]))
}

/// Refused before any connection, so no server has to exist.
fn refusal(result: Result<Response<Body>, RequestError>) -> String {
    match result {
        Err(RequestError::NoAnswer(words)) => words,
        other => panic!("no refusal: {other:?}"),
    }
}

#[test]
fn a_get_over_plain_http_is_refused() {
    let words = refusal(https().get("http://127.0.0.1:9/", &[]));

    assert!(words.contains("https only"), "{words}");
}

#[test]
fn a_post_over_plain_http_is_refused() {
    let words = refusal(https().post("http://127.0.0.1:9/", &[], "{}"));

    assert!(words.contains("https only"), "{words}");
}

#[test]
fn a_text_below_the_cap_is_read() {
    let below = usize::try_from(CAP - 1).unwrap();

    assert_eq!(https().text(answer(below)), Ok("a".repeat(below)));
}

#[test]
fn a_text_over_the_cap_is_unreadable() {
    let over = usize::try_from(CAP + 1).unwrap();

    assert!(matches!(
        https().text(answer(over)),
        Err(RequestError::Unreadable(_))
    ));
}

#[test]
fn a_reader_stops_at_the_cap() {
    let over = usize::try_from(CAP + 1).unwrap();
    let mut body = Vec::new();

    let read = https().reader(answer(over)).read_to_end(&mut body);

    assert!(read.is_err(), "read {} bytes", body.len());
}

#[test]
fn a_reader_hands_on_a_body_within_the_cap() {
    let mut body = Vec::new();

    https().reader(answer(3)).read_to_end(&mut body).unwrap();

    assert_eq!(body, b"aaa");
}

#[test]
fn a_status_the_server_sent_stays_a_status() {
    assert_eq!(
        crate::from_call(&ureq::Error::StatusCode(404)),
        RequestError::Status(404)
    );
}

#[test]
fn the_system_clock_is_past_2024() {
    assert!(SystemClock.now() > 1_704_067_200);
}
