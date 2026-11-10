//! The sentence of one queued change. `toolbox-core` names a change by a
//! code and its values in order. The Fluent message `change-<code>` takes
//! them as named arguments, so translators see what each one is.

#[cfg(test)]
mod tests;

use fluent_templates::LanguageIdentifier;
use toolbox_core::Summary;

use super::text_with_in;

pub fn sentence_in(language: &LanguageIdentifier, summary: &Summary) -> String {
    let args: Vec<(&'static str, &str)> = summary
        .code
        .argument_names()
        .iter()
        .copied()
        .zip(summary.values.iter().map(String::as_str))
        .collect();

    text_with_in(
        language,
        &format!("change-{}", summary.code.as_str()),
        &args,
    )
    .into_owned()
}
