//! The sentence of one queued change. `toolbox-core` names a change by a
//! code and its values in order. The Fluent message `change-<code>` takes
//! them as named arguments, so translators see what each one is.

use fluent_templates::LanguageIdentifier;
use toolbox_core::Summary;

use super::text_with_in;

pub fn sentence_in(language: &LanguageIdentifier, summary: &Summary) -> String {
    let args: Vec<(&'static str, &str)> = argument_names(summary.code)
        .iter()
        .copied()
        .zip(summary.values.iter().map(String::as_str))
        .collect();

    text_with_in(language, &format!("change-{}", summary.code), &args).into_owned()
}

/// The names of a code's values, in their order. A code with one value
/// calls it `name`.
const fn argument_names(code: &str) -> &'static [&'static str] {
    match code.as_bytes() {
        b"group-join" | b"group-leave" => &["user", "group"],
        b"full-name-set" => &["user", "name"],
        b"home-folder" => &["user", "folder"],
        b"login-shell" => &["user", "shell"],
        b"dkms-build" => &["module", "kernel"],
        _ => &["name"],
    }
}
