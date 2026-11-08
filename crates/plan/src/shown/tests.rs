use crate::protocol::{PROTOCOL_VERSION, Request, RequestError};
use crate::{
    ConfigFilePath, EditCopy, HomePath, InvalidConfigFilePath, InvalidEditCopy, InvalidHomePath,
    InvalidShellPath, ShellPath,
};

const ESCAPE: &str = "\u{1b}]52;c;aGk=\u{7}\u{1b}[2J";

/// What every refusal of a value must look like: no raw control
/// character, and a short start of the value instead of all of it.
fn assert_shown(message: &str) {
    assert!(
        !message.chars().any(char::is_control),
        "a control character in {message:?}"
    );
    assert!(message.chars().count() < 300, "too long: {message}");
}

#[test]
fn a_control_character_is_found_before_a_path_is_called_relative() {
    let text = format!("etc/{ESCAPE}");

    assert_eq!(
        ConfigFilePath::new(text.as_str()).unwrap_err(),
        InvalidConfigFilePath::InvalidCharacter('\u{1b}')
    );
    assert_eq!(
        EditCopy::new(text.as_str()).unwrap_err(),
        InvalidEditCopy::InvalidCharacter('\u{1b}')
    );
    assert_eq!(
        HomePath::new(format!("/{text}")).unwrap_err(),
        InvalidHomePath::InvalidCharacter('\u{1b}')
    );
    assert_eq!(
        ShellPath::new(text.as_str()).unwrap_err(),
        InvalidShellPath::InvalidCharacter('\u{1b}')
    );
}

#[test]
fn an_overlong_refused_value_is_cut() {
    let long = "a".repeat(1000);

    for message in [
        ConfigFilePath::new(long.as_str()).unwrap_err().to_string(),
        ConfigFilePath::new(format!("/var/{long}"))
            .unwrap_err()
            .to_string(),
        EditCopy::new(long.as_str()).unwrap_err().to_string(),
        HomePath::new(format!("/{long}")).unwrap_err().to_string(),
        ShellPath::new("a".repeat(200)).unwrap_err().to_string(),
    ] {
        assert_shown(&message);
        assert!(message.contains("aaaa…"), "{message}");
    }
}

#[test]
fn a_component_with_an_escape_sequence_is_spelled_out() {
    let folder = ConfigFilePath::new("/etc/modprobe.d").unwrap();
    let error = folder.join(&format!("{ESCAPE}/x")).unwrap_err();

    assert!(matches!(error, InvalidConfigFilePath::InvalidComponent(_)));
    assert_shown(&error.to_string());
    assert!(error.to_string().contains("\\u{1b}"), "{error}");
}

#[test]
fn a_malformed_request_quotes_only_the_start_of_the_error() {
    let value = format!("{ESCAPE}{}", "b".repeat(100_000));
    let line = format!(
        "{{\"version\":{PROTOCOL_VERSION},\"mode\":\"run\",\"steps\":[{{\"id\":1,\"action\":{{\"kind\":{}}}}}]}}",
        serde_json::to_string(&value).unwrap()
    );

    let error = Request::parse(&line).unwrap_err();

    assert!(matches!(error, RequestError::Malformed(_)), "{error}");
    assert_shown(&error.to_string());
}
