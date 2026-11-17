//! A D-Bus value as JSON, the shape busctl prints without its types.
//! zbus serializes a variant with its signature beside the value, which
//! the typed reads do not expect.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use serde_json::{Map, Value};
use zbus::{OwnedVariant, Variant};

/// The reply of `GetAll`: each property by its name.
pub fn properties_json(properties: &HashMap<String, OwnedVariant>) -> Value {
    Value::Object(
        properties
            .iter()
            .map(|(name, value)| (name.clone(), json(value)))
            .collect(),
    )
}

pub fn json(value: &Variant<'_>) -> Value {
    match value {
        Variant::U8(number) => Value::from(*number),
        Variant::Bool(truth) => Value::Bool(*truth),
        Variant::I16(number) => Value::from(*number),
        Variant::U16(number) => Value::from(*number),
        Variant::I32(number) => Value::from(*number),
        Variant::U32(number) => Value::from(*number),
        Variant::I64(number) => Value::from(*number),
        Variant::U64(number) => Value::from(*number),
        Variant::F64(number) => Value::from(*number),
        Variant::Str(text) => Value::from(text.as_str()),
        Variant::Signature(signature) => Value::from(signature.to_string()),
        Variant::ObjectPath(path) => Value::from(path.as_str()),
        Variant::Variant(inner) => json(inner),
        Variant::Array(items) => items.inner().iter().map(json).collect(),
        Variant::Dict(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, item)| (key_text(key), json(item)))
                .collect::<Map<_, _>>(),
        ),
        Variant::Structure(fields) => fields.fields().iter().map(json).collect(),
        // A file descriptor means nothing outside the process it was sent to.
        Variant::Fd(_) => Value::Null,
    }
}

/// A JSON key: a string as it is, any other key as JSON text.
fn key_text(key: &Variant<'_>) -> String {
    match json(key) {
        Value::String(text) => text,
        other @ (Value::Null
        | Value::Bool(_)
        | Value::Number(_)
        | Value::Array(_)
        | Value::Object(_)) => other.to_string(),
    }
}
