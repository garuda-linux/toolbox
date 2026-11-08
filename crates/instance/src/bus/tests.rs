use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use zbus::fdo::RequestNameReply;
use zbus::{OwnedVariant, Variant};

use super::{Application, Arrivals, BUS_LIMIT, Bus, Start, owned, settle};
use crate::activation::{Activation, Token};

#[derive(Default)]
struct Kept(Mutex<Vec<Activation>>);

impl Arrivals for Kept {
    fn arrive(&self, activation: Activation) {
        self.0.lock().push(activation);
    }
}

struct Fake {
    claim: Option<Result<bool, String>>,
    hand_over: Option<Result<(), String>>,
    handed: Mutex<Vec<Activation>>,
}

impl Fake {
    fn new(claim: Option<Result<bool, String>>, hand_over: Option<Result<(), String>>) -> Self {
        Self {
            claim,
            hand_over,
            handed: Mutex::new(Vec::new()),
        }
    }
}

impl Bus for Fake {
    type Held = &'static str;
    type Error = String;

    async fn claim(&self) -> Result<Option<Self::Held>, Self::Error> {
        match self.claim.clone() {
            Some(answer) => answer.map(|first| first.then_some("held")),
            None => std::future::pending().await,
        }
    }

    async fn hand_over(&self, activation: &Activation) -> Result<(), Self::Error> {
        self.handed.lock().push(activation.clone());
        match self.hand_over.clone() {
            Some(answer) => answer,
            None => std::future::pending().await,
        }
    }
}

fn string(text: &str) -> OwnedVariant {
    OwnedVariant::try_from(Variant::from(text)).unwrap()
}

fn tokens() -> HashMap<String, OwnedVariant> {
    HashMap::from([
        ("activation-token".to_owned(), string("wayland-token")),
        ("desktop-startup-id".to_owned(), string("x11-id")),
        ("unknown".to_owned(), OwnedVariant::from(7_u32)),
    ])
}

fn application() -> (Application, Arc<Kept>) {
    let kept = Arc::new(Kept::default());
    let arrivals: Arc<dyn Arrivals> = Arc::<Kept>::clone(&kept);
    (Application { arrivals }, kept)
}

#[tokio::test]
async fn the_first_start_holds_the_name() {
    let bus = Fake::new(Some(Ok(true)), None);

    assert!(matches!(
        settle(&bus, &Activation::default(), BUS_LIMIT).await,
        Start::First("held")
    ));
    assert!(bus.handed.lock().is_empty());
}

#[tokio::test]
async fn a_later_start_hands_its_wish_over() {
    let bus = Fake::new(Some(Ok(false)), Some(Ok(())));
    let wish = Activation::from_action("open-page", &["maintenance"]).unwrap();

    assert!(matches!(
        settle(&bus, &wish, BUS_LIMIT).await,
        Start::Handed
    ));
    assert_eq!(*bus.handed.lock(), [wish]);
}

#[tokio::test]
async fn a_failed_hand_over_or_no_bus_starts_alone() {
    let refused = Fake::new(Some(Ok(false)), Some(Err("no such interface".to_owned())));
    let no_bus = Fake::new(Some(Err("no session bus".to_owned())), None);

    assert!(matches!(
        settle(&refused, &Activation::default(), BUS_LIMIT).await,
        Start::Alone
    ));
    assert!(matches!(
        settle(&no_bus, &Activation::default(), BUS_LIMIT).await,
        Start::Alone
    ));
}

#[tokio::test(start_paused = true)]
async fn a_bus_or_an_app_that_hangs_starts_alone() {
    let silent_bus = Fake::new(None, None);
    let silent_app = Fake::new(Some(Ok(false)), None);

    assert!(matches!(
        settle(&silent_bus, &Activation::default(), BUS_LIMIT).await,
        Start::Alone
    ));
    assert!(matches!(
        settle(&silent_app, &Activation::default(), BUS_LIMIT).await,
        Start::Alone
    ));
}

#[test]
fn activate_brings_the_tokens_and_no_page() {
    let (application, kept) = application();

    application.activate(tokens());

    let arrived = kept.0.lock().clone();
    assert_eq!(arrived.len(), 1);
    assert_eq!(arrived[0].page, None);
    assert_eq!(
        arrived[0].token.as_ref().map(Token::as_str),
        Some("wayland-token")
    );
    assert_eq!(
        arrived[0].startup_id.as_ref().map(Token::as_str),
        Some("x11-id")
    );
}

#[test]
fn activate_action_opens_a_page_with_its_action() {
    let (application, kept) = application();

    application
        .activate_action(
            "open-page",
            vec![string("diagnostics"), string("inxi")],
            tokens(),
        )
        .unwrap();

    let arrived = kept.0.lock().clone();
    assert_eq!(
        arrived[0].page.as_ref().map(AsRef::as_ref),
        Some("diagnostics")
    );
    assert_eq!(arrived[0].action.as_ref().map(AsRef::as_ref), Some("inxi"));
    assert!(arrived[0].token.is_some());
}

#[test]
fn a_wrong_action_or_parameter_is_an_error_and_nothing_arrives() {
    let (application, kept) = application();

    assert!(
        application
            .activate_action("format-disk", Vec::new(), HashMap::new())
            .is_err()
    );
    assert!(
        application
            .activate_action("open-page", vec![OwnedVariant::from(1_u32)], HashMap::new())
            .is_err()
    );
    assert!(
        application
            .open(vec!["file:///etc/passwd".to_owned()], HashMap::new())
            .is_err()
    );
    assert!(kept.0.lock().is_empty());
}

#[test]
fn a_taken_name_means_another_start_holds_it() {
    assert!(matches!(
        owned(Ok(RequestNameReply::PrimaryOwner)),
        Ok(true)
    ));
    assert!(matches!(owned(Ok(RequestNameReply::Exists)), Ok(false)));
    assert!(matches!(owned(Err(zbus::Error::NameTaken)), Ok(false)));
    drop(owned(Err(zbus::Error::InvalidReply)).unwrap_err());
}
