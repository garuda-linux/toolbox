//! Statuses through ureq's whole call, on a transport that answers fixed
//! bytes. No server sends a redirect or a `304` on demand, and a local TLS
//! server would need a certificate the platform verifier trusts.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ureq::unversioned::transport::{
    Buffers, ConnectionDetails, Connector, LazyBuffers, NextTimeout, Transport,
};

use super::RULES;
use crate::{Https, RequestError};

const URL: &str = "https://127.0.0.1/feed";

/// Hands out one [`Canned`] per connection and counts them, so a followed
/// redirect would show as a second one.
#[derive(Debug)]
struct Server {
    answer: &'static [u8],
    connections: Arc<AtomicUsize>,
}

/// Ignores the request and answers `answer`, then the end of the stream.
#[derive(Debug)]
struct Canned {
    buffers: LazyBuffers,
    answer: &'static [u8],
}

impl Connector for Server {
    type Out = Canned;

    fn connect(
        &self,
        _details: &ConnectionDetails<'_>,
        _chained: Option<()>,
    ) -> Result<Option<Canned>, ureq::Error> {
        self.connections.fetch_add(1, Ordering::SeqCst);

        Ok(Some(Canned {
            buffers: LazyBuffers::new(16 << 10, 16 << 10),
            answer: self.answer,
        }))
    }
}

impl Transport for Canned {
    fn buffers(&mut self) -> &mut dyn Buffers {
        &mut self.buffers
    }

    fn transmit_output(
        &mut self,
        _amount: usize,
        _timeout: NextTimeout,
    ) -> Result<(), ureq::Error> {
        Ok(())
    }

    fn await_input(&mut self, _timeout: NextTimeout) -> Result<bool, ureq::Error> {
        let input = self.buffers.input_append_buf();
        let amount = self.answer.len().min(input.len());
        input[..amount].copy_from_slice(&self.answer[..amount]);
        self.buffers.input_appended(amount);
        self.answer = &self.answer[amount..];

        Ok(amount > 0)
    }

    fn is_open(&mut self) -> bool {
        false
    }

    // ureq refuses a plain connection for an https URL.
    fn is_tls(&self) -> bool {
        true
    }
}

fn serving(answer: &'static [u8]) -> (Https, Arc<AtomicUsize>) {
    let connections = Arc::new(AtomicUsize::new(0));
    let server = Server {
        answer,
        connections: Arc::clone(&connections),
    };

    (Https::with_connector(&RULES, server), connections)
}

#[test]
fn a_redirect_is_a_status_and_not_followed() {
    let (https, connections) = serving(
        b"HTTP/1.1 301 Moved Permanently\r\nLocation: https://127.0.0.1/elsewhere\r\nContent-Length: 0\r\n\r\n",
    );

    let answer = https.get(URL, &[]);

    assert_eq!(answer.err(), Some(RequestError::Status(301)));
    assert_eq!(connections.load(Ordering::SeqCst), 1);
}

#[test]
fn a_redirect_after_a_post_is_a_status() {
    let (https, _) = serving(
        b"HTTP/1.1 302 Found\r\nLocation: https://127.0.0.1/elsewhere\r\nContent-Length: 0\r\n\r\n",
    );

    assert_eq!(
        https.post(URL, &[], "{}").err(),
        Some(RequestError::Status(302))
    );
}

#[test]
fn not_modified_is_a_status() {
    let (https, _) = serving(b"HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n");

    assert_eq!(https.get(URL, &[]).err(), Some(RequestError::Status(304)));
}

#[test]
fn a_server_error_is_a_status() {
    let (https, _) = serving(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");

    assert_eq!(https.get(URL, &[]).err(), Some(RequestError::Status(503)));
}

#[test]
fn a_2xx_is_an_answer() {
    let (https, _) = serving(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc");

    let response = https.get(URL, &[]).unwrap();

    assert_eq!(https.text(response), Ok("abc".to_owned()));
}
