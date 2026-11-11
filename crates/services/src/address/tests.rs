use std::net::IpAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

use toolbox_https::RequestError;

use super::{ADDRESS_HOST, ADDRESS_SERVICE, AddressProblem, AddressSource, lookup_address};

struct FakeService {
    answer: Result<&'static str, &'static str>,
    asked: AtomicUsize,
}

impl FakeService {
    const fn new(answer: Result<&'static str, &'static str>) -> Self {
        Self {
            answer,
            asked: AtomicUsize::new(0),
        }
    }
}

impl AddressSource for FakeService {
    fn answer(&self) -> Result<String, RequestError> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        self.answer
            .map(str::to_owned)
            .map_err(|reason| RequestError::NoAnswer(reason.to_owned()))
    }
}

#[test]
fn one_request_gives_the_address_and_nothing_else_of_the_answer() {
    let service = FakeService::new(Ok(
        "<Response><Ip>198.51.100.23</Ip><Status>OK</Status><CountryCode>DE</CountryCode>\
         <City>Berlin</City><Latitude>52.5</Latitude></Response>",
    ));

    assert_eq!(
        lookup_address(&service),
        Ok("198.51.100.23".parse::<IpAddr>().unwrap())
    );
    assert_eq!(service.asked.load(Ordering::SeqCst), 1);
}

#[test]
fn no_answer_is_reported_with_its_reason() {
    let service = FakeService::new(Err("io: Network is unreachable"));

    assert_eq!(
        lookup_address(&service),
        Err(AddressProblem::NoAnswer(RequestError::NoAnswer(
            "io: Network is unreachable".to_owned()
        )))
    );
}

#[test]
fn an_answer_without_an_address_is_not_shown() {
    for answer in [
        "",
        "<html><h1>Service unavailable</h1></html>",
        "<Response><Ip>&lt;script&gt;</Ip></Response>",
    ] {
        assert_eq!(
            lookup_address(&FakeService::new(Ok(answer))),
            Err(AddressProblem::NotUnderstood),
            "{answer}"
        );
    }
}

#[test]
fn the_pinned_host_is_the_services() {
    let url: toolbox_https::http::Uri = ADDRESS_SERVICE.parse().unwrap();

    assert_eq!(url.host(), Some(ADDRESS_HOST));
}
