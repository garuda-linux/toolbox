use super::{is_on, set};
use crate::EditError;

const KEY: &str = "SKIP_HOST_UPDATE";

#[test]
fn on_means_the_key_is_true() {
    assert!(is_on("{\"SKIP_HOST_UPDATE\": true}", KEY));
    assert!(!is_on("{\"SKIP_HOST_UPDATE\": false}", KEY));
    assert!(!is_on("{\"SKIP_HOST_UPDATE\": \"true\"}", KEY));
    assert!(!is_on("{}", KEY));
    assert!(!is_on("not json", KEY));
    assert!(!is_on("[true]", KEY));
}

#[test]
fn turning_on_keeps_the_other_keys() {
    let after = set(
        "{\"BACKGROUND_COLOR\": \"#202225\", \"zoom\": 1.5}",
        KEY,
        true,
    )
    .unwrap()
    .unwrap();

    let value: serde_json::Value = serde_json::from_str(&after).unwrap();

    assert_eq!(
        value,
        serde_json::json!({"BACKGROUND_COLOR": "#202225", "zoom": 1.5_f64, "SKIP_HOST_UPDATE": true})
    );
    assert!(after.ends_with('\n'));
}

#[test]
fn turning_off_removes_only_the_key() {
    let after = set("{\"SKIP_HOST_UPDATE\": true, \"zoom\": 1}", KEY, false)
        .unwrap()
        .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&after).unwrap(),
        serde_json::json!({"zoom": 1_i32})
    );
}

#[test]
fn a_missing_or_blank_file_is_an_empty_object() {
    assert_eq!(
        set("", KEY, true).unwrap().unwrap(),
        "{\n  \"SKIP_HOST_UPDATE\": true\n}\n"
    );
    assert_eq!(set(" \n", KEY, false).unwrap(), None);
}

#[test]
fn a_file_already_as_wanted_is_left_alone() {
    assert_eq!(
        set("{\"SKIP_HOST_UPDATE\": true}", KEY, true).unwrap(),
        None
    );
    assert_eq!(set("{\"zoom\": 1}", KEY, false).unwrap(), None);
}

#[test]
fn a_false_flag_turns_true() {
    assert_eq!(
        set("{\"SKIP_HOST_UPDATE\": false}", KEY, true).unwrap(),
        Some("{\n  \"SKIP_HOST_UPDATE\": true\n}\n".to_owned())
    );
}

/// The file is the user's or an app's: an edit of one key keeps the order
/// of the others, so the diff shows that key alone.
#[test]
fn the_key_order_survives_an_edit() {
    let text = "{\"zoom\": 1, \"BACKGROUND_COLOR\": \"#000\", \"a\": {\"z\": 1, \"b\": 2}}";

    assert_eq!(
        set(text, KEY, true).unwrap(),
        Some(
            "{\n  \"zoom\": 1,\n  \"BACKGROUND_COLOR\": \"#000\",\n  \"a\": {\n    \"z\": 1,\n    \"b\": 2\n  },\n  \"SKIP_HOST_UPDATE\": true\n}\n"
                .to_owned()
        )
    );

    let flagged =
        "{\"zoom\": 1, \"SKIP_HOST_UPDATE\": true, \"BACKGROUND_COLOR\": \"#000\", \"a\": 2}";

    assert_eq!(
        set(flagged, KEY, false).unwrap(),
        Some("{\n  \"zoom\": 1,\n  \"BACKGROUND_COLOR\": \"#000\",\n  \"a\": 2\n}\n".to_owned())
    );
}

#[test]
fn refuses_what_is_no_json_object() {
    assert_eq!(set("[1, 2]", KEY, true), Err(EditError::NotJsonObject));
    assert!(matches!(
        set("{\"zoom\": ", KEY, true),
        Err(EditError::InvalidJson(_))
    ));
}
