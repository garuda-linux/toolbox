//! One `true` flag at the top level of a JSON object, such as Discord's
//! `SKIP_HOST_UPDATE`. Every other key stays.

#[cfg(test)]
mod tests;

use serde_json::{Map, Value};

use super::EditError;

/// Whether the object in `text` has `key` set to `true`. Anything that is
/// no JSON object counts as off.
pub fn is_on(text: &str, key: &str) -> bool {
    object(text).is_ok_and(|object| object.get(key) == Some(&Value::Bool(true)))
}

/// `text` with `key` set to `true` (`on`) or removed, `None` when it already
/// is that way. A missing or blank file counts as an empty object.
///
/// # Errors
///
/// [`EditError::InvalidJson`] or [`EditError::NotJsonObject`]: a file the
/// edit cannot understand stays as it is.
pub fn set(text: &str, key: &str, on: bool) -> Result<Option<String>, EditError> {
    let mut object = object(text)?;

    let changed = if on {
        object.insert(key.to_owned(), Value::Bool(true)) != Some(Value::Bool(true))
    } else {
        // `remove` would move the last key into the gap.
        object.shift_remove(key).is_some()
    };

    if !changed {
        return Ok(None);
    }

    let mut out = serde_json::to_string_pretty(&Value::Object(object))
        .map_err(|error| EditError::InvalidJson(error.to_string()))?;

    out.push('\n');
    Ok(Some(out))
}

fn object(text: &str) -> Result<Map<String, Value>, EditError> {
    if text.trim().is_empty() {
        return Ok(Map::new());
    }

    let value: Value =
        serde_json::from_str(text).map_err(|error| EditError::InvalidJson(error.to_string()))?;

    match value {
        Value::Object(object) => Ok(object),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Array(_) => {
            Err(EditError::NotJsonObject)
        }
    }
}
