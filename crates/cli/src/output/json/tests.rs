use serde::Serialize;
use serde::ser::Error as _;

use super::{json, json_line, with_schema};

#[derive(Serialize)]
struct Row {
    id: &'static str,
    on: bool,
}

struct Broken;

impl Serialize for Broken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        drop(serializer);
        Err(S::Error::custom("no value"))
    }
}

#[test]
fn an_object_starts_with_its_schema() {
    let text = json(&Row {
        id: "zram",
        on: true,
    });

    assert_eq!(
        text,
        "{\n  \"schema\": 1,\n  \"id\": \"zram\",\n  \"on\": true\n}\n"
    );
}

#[test]
fn a_line_is_one_compact_object() {
    let text = json_line(&Row {
        id: "zram",
        on: false,
    });

    assert_eq!(text, "{\"schema\":1,\"id\":\"zram\",\"on\":false}\n");
}

/// A list is an object too, so a script reads `schema` from every output
/// alike, also with one row or none.
#[test]
fn a_list_carries_its_rows_behind_the_schema() {
    let rows = [Row {
        id: "zram",
        on: true,
    }];

    assert_eq!(
        json_line(&rows),
        "{\"schema\":1,\"rows\":[{\"id\":\"zram\",\"on\":true}]}\n"
    );
    assert_eq!(
        json_line(&Vec::<Row>::new()),
        "{\"schema\":1,\"rows\":[]}\n"
    );
}

#[test]
fn a_plain_value_is_carried_as_value() {
    assert_eq!(
        json_line(&"192.0.2.1"),
        "{\"schema\":1,\"value\":\"192.0.2.1\"}\n"
    );
}

/// The fallback is JSON too, so a script that parses it never meets text.
#[test]
fn a_value_that_does_not_serialise_becomes_an_error_object() {
    let shaped = with_schema(&Broken);

    assert_eq!(shaped["schema"], 1_u64);
    assert_eq!(shaped["error"]["message"], "no value");
}
