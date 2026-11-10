use toolbox_core::features::diagnostics::{Collector, Selection};
use toolbox_instance::activation::{Activation, Token};
use toolbox_instance::bus::Arrivals as _;

use super::{Arrival, Arrivals, PageAction};
use crate::command::{self, AppCommand, Command};
use crate::page::PageId;

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
fn a_token_still_leaves_the_app_to_ask_for_attention() {
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

/// Any process of the session may call. A collection may ask for a
/// password, so it starts only for a window the user has in front.
#[test]
fn a_collection_starts_only_in_a_window_in_front() {
    let arrival = Arrival::of(&wish("diagnostics", Some("inxi")), true);
    let inxi = Some(Selection::One(Collector::Inxi));

    assert_eq!(arrival.collection(true), inxi);
    assert_eq!(arrival.collection(false), None);
    assert_eq!(
        Arrival::of(&wish("diagnostics", None), false).collection(true),
        None
    );
}
