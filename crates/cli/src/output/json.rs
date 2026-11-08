//! JSON for scripts. Every object carries `schema` first, the version of
//! its field names, so a script can tell a later shape from this one.

#[cfg(test)]
mod tests;

use serde::Serialize;
use serde_json::{Map, Value};

/// The version of the field names `--json` prints.
pub const SCHEMA: u64 = 1;

/// Pretty JSON with a final newline. A value of the app always serialises.
pub fn json<T>(value: &T) -> String
where
    T: Serialize,
{
    let shaped = with_schema(value);
    let mut text = serde_json::to_string_pretty(&shaped).unwrap_or_else(|_unencodable| {
        // Only a map with keys that are no text fails, and a value holds none.
        "{}".to_owned()
    });

    text.push('\n');
    text
}

/// One line of JSON Lines, with its line break.
pub fn json_line<T>(value: &T) -> String
where
    T: Serialize,
{
    let shaped = with_schema(value);
    let mut text = serde_json::to_string(&shaped).unwrap_or_else(|_unencodable| "{}".to_owned());

    text.push('\n');
    text
}

/// `value` as a JSON object with `schema` in front of its own fields.
///
/// An array becomes the object's `rows`, any other value its `value`. A
/// value that does not serialise becomes an error object, never text that
/// is no JSON.
pub fn with_schema<T>(value: &T) -> Value
where
    T: Serialize,
{
    match serde_json::to_value(value) {
        Ok(Value::Object(fields)) => {
            let mut shaped = Map::with_capacity(fields.len() + 1);
            shaped.insert("schema".to_owned(), Value::from(SCHEMA));
            shaped.extend(fields.into_iter().filter(|(key, _)| key != "schema"));
            Value::Object(shaped)
        }
        Ok(rows @ Value::Array(_)) => serde_json::json!({ "schema": SCHEMA, "rows": rows }),
        Ok(other) => serde_json::json!({ "schema": SCHEMA, "value": other }),
        Err(error) => serde_json::json!({
            "schema": SCHEMA,
            "error": { "message": error.to_string() },
        }),
    }
}
