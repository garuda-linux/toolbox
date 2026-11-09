//! The one way the app talks HTTPS: blocking ureq with rustls, checked
//! against the system's certificates. Every agent talks to its one host,
//! refuses plain HTTP and every redirect, gives up after a global timeout
//! and reads no more than its cap. A caller picks the host, the timeout,
//! the cap and the user agent.
//!
//! [`Clock`] lives here too, so every cache of a download reads the time
//! the same way.

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ureq::Agent;
use ureq::config::Config;
use ureq::http::{HeaderName, Response, Uri};
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
pub use ureq::{Body, BodyReader, http};

/// The only port an agent reaches.
const HTTPS_PORT: u16 = 443;

/// What sets one caller's agent apart from another's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// The one server this agent talks to. A URL of another is refused
    /// before any connection.
    pub host: &'static str,
    /// For the whole request, from the connection to the last byte read.
    pub timeout: Duration,
    /// Past this the body is refused instead of read on.
    pub max_body_bytes: u64,
    /// `None` sends ureq's own.
    pub user_agent: Option<&'static str>,
}

/// Why a request got no answer to read. The words are the HTTP client's,
/// for the log and a message's details. Never a part of an answer.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RequestError {
    /// The server was not reached or did not answer in time: no network,
    /// DNS, TLS, a timeout, plain HTTP. The news feed puts a body past its
    /// cap here too.
    #[error("{0}")]
    NoAnswer(String),
    /// A URL of another host or port than the agent's, refused before any
    /// connection.
    #[error("{0} is not on the one host of this agent")]
    OtherHost(String),
    /// Any status but 2xx: a redirect and `304 Not Modified` too.
    #[error("the server answered with status {0}")]
    Status(u16),
    /// The answer could not be read, or it is larger than allowed.
    #[error("the answer could not be read: {0}")]
    Unreadable(String),
}

/// Answers what time it is, in seconds since the epoch.
pub trait Clock: Send + Sync {
    fn now(&self) -> i64;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

/// Blocks while a request runs, so call it off the GUI thread.
pub struct Https {
    agent: Agent,
    host: &'static str,
    max_body_bytes: u64,
}

impl Https {
    #[must_use = "the agent does nothing until a request is sent"]
    pub fn new(rules: &Rules) -> Self {
        Self {
            agent: config(rules).new_agent(),
            host: rules.host,
            max_body_bytes: rules.max_body_bytes,
        }
    }

    /// Lets tests answer with statuses no real server sends on demand.
    #[cfg(test)]
    fn with_connector(
        rules: &Rules,
        connector: impl ureq::unversioned::transport::Connector,
    ) -> Self {
        Self {
            agent: Agent::with_parts(
                config(rules),
                connector,
                ureq::unversioned::resolver::DefaultResolver::default(),
            ),
            host: rules.host,
            max_body_bytes: rules.max_body_bytes,
        }
    }

    /// # Errors
    ///
    /// [`RequestError`] when the URL is of another host, the server was
    /// not reached or sent a status other than 2xx.
    pub fn get(
        &self,
        url: &str,
        headers: &[(HeaderName, &str)],
    ) -> Result<Response<Body>, RequestError> {
        self.check_host(url)?;
        let mut request = self.agent.get(url);
        for (name, value) in headers {
            request = request.header(name, *value);
        }

        answered(request.call())
    }

    /// # Errors
    ///
    /// [`RequestError`] when the URL is of another host, the server was
    /// not reached or sent a status other than 2xx.
    pub fn post(
        &self,
        url: &str,
        headers: &[(HeaderName, &str)],
        body: &str,
    ) -> Result<Response<Body>, RequestError> {
        self.check_host(url)?;
        let mut request = self.agent.post(url);
        for (name, value) in headers {
            request = request.header(name, *value);
        }

        answered(request.send(body))
    }

    /// The body, failing at the first byte past the cap.
    pub fn reader(&self, response: Response<Body>) -> BodyReader<'static> {
        response
            .into_body()
            .into_with_config()
            .limit(self.max_body_bytes)
            .reader()
    }

    /// # Errors
    ///
    /// [`RequestError::Unreadable`] for a body past the cap or not UTF-8.
    pub fn text(&self, mut response: Response<Body>) -> Result<String, RequestError> {
        response
            .body_mut()
            .with_config()
            .limit(self.max_body_bytes)
            .read_to_string()
            .map_err(|error| RequestError::Unreadable(error.to_string()))
    }

    /// Every address is a constant, so another host is a mistake in the
    /// code. It is refused rather than reached.
    fn check_host(&self, url: &str) -> Result<(), RequestError> {
        let uri: Uri = url
            .parse()
            .map_err(|error: http::uri::InvalidUri| RequestError::NoAnswer(error.to_string()))?;

        let host = uri
            .host()
            .is_some_and(|host| host.eq_ignore_ascii_case(self.host));

        let port = uri.port_u16().is_none_or(|port| port == HTTPS_PORT);

        if host && port {
            Ok(())
        } else {
            Err(RequestError::OtherHost(url.to_owned()))
        }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| {
                i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
            })
    }
}

fn config(rules: &Rules) -> Config {
    let tls = TlsConfig::builder()
        .provider(TlsProvider::Rustls)
        .root_certs(RootCerts::PlatformVerifier)
        .unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .build();

    let config = Agent::config_builder()
        .https_only(true)
        // A redirect would reach a server the user was not told about.
        // It is answered as a failure instead.
        .max_redirects(0)
        .timeout_global(Some(rules.timeout))
        .tls_config(tls);

    match rules.user_agent {
        Some(agent) => config.user_agent(agent),
        None => config,
    }
    .build()
}

/// ureq hands a 3xx back as an answer when it follows no redirect. Only a
/// 2xx counts here, so no caller reads a redirect's body as the content.
fn answered(result: Result<Response<Body>, ureq::Error>) -> Result<Response<Body>, RequestError> {
    let response = result.map_err(|error| from_call(&error))?;
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(RequestError::Status(response.status().as_u16()))
    }
}

/// ureq reports a status as an error, which still means the server answered.
fn from_call(error: &ureq::Error) -> RequestError {
    if let &ureq::Error::StatusCode(status) = error {
        RequestError::Status(status)
    } else {
        RequestError::NoAnswer(error.to_string())
    }
}
