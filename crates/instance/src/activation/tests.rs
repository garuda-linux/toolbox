use super::{Activation, Call, Refusal, Token};

fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |key| {
        pairs
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_owned())
    }
}

#[test]
fn a_token_is_visible_ascii_of_bounded_length() {
    assert!(Token::new("kwin-4711-ab_TIME1234").is_some());
    assert!(Token::new("").is_none());
    assert!(Token::new("with space").is_none());
    assert!(Token::new("line\nbreak").is_none());
    assert!(Token::new("ümlaut").is_none());
    assert!(Token::new(&"t".repeat(1025)).is_none());
    assert!(Token::new(&"t".repeat(1024)).is_some());
}

#[test]
fn a_start_takes_its_tokens_from_the_environment() {
    let start = Activation::from_env(env(&[
        ("XDG_ACTIVATION_TOKEN", "wayland-token"),
        ("DESKTOP_STARTUP_ID", "x11-id_TIME42"),
    ]));

    assert_eq!(
        start.token.as_ref().map(Token::as_str),
        Some("wayland-token")
    );
    assert_eq!(
        start.startup_id.as_ref().map(Token::as_str),
        Some("x11-id_TIME42")
    );
    assert_eq!(start.page, None);

    let broken = Activation::from_env(env(&[("XDG_ACTIVATION_TOKEN", "a b")]));
    assert_eq!(broken.token, None);
}

#[test]
fn platform_data_carries_the_tokens_and_ignores_the_rest() {
    let activation = Activation::default().with_platform_data([
        ("activation-token", Some("wayland-token")),
        ("desktop-startup-id", Some("x11-id")),
        ("something-else", Some("ignored")),
        ("activation-token-2", None),
    ]);

    assert_eq!(
        activation.token.as_ref().map(Token::as_str),
        Some("wayland-token")
    );
    assert_eq!(
        activation.startup_id.as_ref().map(Token::as_str),
        Some("x11-id")
    );
    assert_eq!(
        activation.platform_data(),
        vec![
            ("activation-token", "wayland-token"),
            ("desktop-startup-id", "x11-id")
        ]
    );
}

#[test]
fn open_page_takes_a_page_and_maybe_its_action() {
    let page = Activation::from_action("open-page", &["diagnostics"]).unwrap();
    assert_eq!(page.page.as_ref().map(AsRef::as_ref), Some("diagnostics"));
    assert_eq!(page.action, None);

    let action = Activation::from_action("open-page", &["diagnostics", "inxi"]).unwrap();
    assert_eq!(action.action.as_ref().map(AsRef::as_ref), Some("inxi"));
}

#[test]
fn a_wrong_action_or_parameter_is_refused() {
    assert_eq!(
        Activation::from_action("format-disk", &[]),
        Err(Refusal::UnknownAction)
    );
    assert_eq!(
        Activation::from_action("open-page", &[]),
        Err(Refusal::BadParameter)
    );
    assert_eq!(
        Activation::from_action("open-page", &["Not A Page"]),
        Err(Refusal::BadParameter)
    );
    assert_eq!(
        Activation::from_action("open-page", &["a", "b", "c"]),
        Err(Refusal::BadParameter)
    );
}

#[test]
fn the_call_names_the_page_only_when_there_is_one() {
    assert_eq!(Activation::default().call(), Call::Activate);

    let action = Activation::from_action("open-page", &["diagnostics", "inxi"]).unwrap();
    assert_eq!(
        action.call(),
        Call::ActivateAction {
            name: "open-page",
            parameters: vec!["diagnostics", "inxi"],
        }
    );
}
