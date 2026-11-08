use std::sync::Arc;

use parking_lot::Mutex;
use toolbox_core::features::diagnostics::{Collector, Selection};
use toolbox_instance::activation::{Activation, Token};
use toolbox_instance::bus::Arrivals as _;

use super::{Arrival, Arrivals, PageAction, Raise};
use crate::command::{self, AppCommand, Command};
use crate::page::PageId;

/// Keeps the tokens it was asked to raise with.
#[derive(Default)]
struct Raised(Mutex<Vec<String>>);

impl Raise for Raised {
    fn raise(&self, token: &Token) {
        self.0.lock().push(token.as_str().to_owned());
    }
}

fn wish(page: &str, action: Option<&str>) -> Activation {
    let mut parameters = vec![page];
    parameters.extend(action);
    Activation::from_action("open-page", &parameters).unwrap()
}

#[test]
fn an_arrival_names_a_known_page_and_its_collector() {
    let arrival = Arrival::of(&wish("diagnostics", Some("inxi")), false);

    assert_eq!(arrival.page, Some(PageId::Diagnostics));
    assert_eq!(
        arrival.action,
        Some(PageAction::Collect(Selection::One(Collector::Inxi)))
    );
}

#[test]
fn an_unknown_page_or_action_is_dropped_but_the_window_still_comes() {
    let unknown = Arrival::of(&wish("review", None), false);
    assert_eq!(unknown.page, None);

    let no_collector = Arrival::of(&wish("diagnostics", Some("rm-rf")), false);
    assert_eq!(no_collector.page, Some(PageId::Diagnostics));
    assert_eq!(no_collector.action, None);

    let elsewhere = Arrival::of(&wish("maintenance", Some("inxi")), false);
    assert_eq!(elsewhere.action, None, "only Diagnostics has actions");
}

#[test]
fn a_token_raises_the_window_before_the_app_hears_of_it() {
    let (sender, receiver) = command::channel();
    let raised = Arc::new(Raised::default());
    let arrivals = Arrivals::new(sender);
    arrivals.set_raise(Arc::<Raised>::clone(&raised));

    let mut activation = wish("maintenance", None);
    activation.token = Token::new("wayland-token");
    arrivals.arrive(activation);
    arrivals.arrive(Activation::default());

    assert_eq!(*raised.0.lock(), ["wayland-token"]);

    let heard: Vec<Command> = receiver.drain().map(|sent| sent.command).collect();
    assert_eq!(
        heard,
        [
            Command::App(AppCommand::Activated(Arrival {
                page: Some(PageId::Maintenance),
                action: None,
                raised: true,
            })),
            Command::App(AppCommand::Activated(Arrival {
                page: None,
                action: None,
                raised: false,
            })),
        ]
    );
}

#[test]
fn without_a_raiser_the_app_is_told_to_ask_for_attention() {
    let (sender, receiver) = command::channel();
    let arrivals = Arrivals::new(sender);

    arrivals.arrive(Activation {
        token: Token::new("wayland-token"),
        ..Activation::default()
    });

    let heard: Vec<Command> = receiver.drain().map(|sent| sent.command).collect();
    assert_eq!(
        heard,
        [Command::App(AppCommand::Activated(Arrival {
            page: None,
            action: None,
            raised: false,
        }))]
    );
}
