//! The sentence of one queued change. `toolbox-core` names a change by a
//! code and its values in order. The Fluent message `change-<code>` takes
//! them as named arguments, so translators see what each one is.

#[cfg(test)]
mod tests;

use fluent_templates::LanguageIdentifier;
use toolbox_core::Summary;

use super::{Arg, text_with_in};

pub fn sentence_in(language: &LanguageIdentifier, summary: &Summary) -> String {
    let args: Vec<(&'static str, Arg<'_>)> = summary
        .code
        .argument_names()
        .iter()
        .copied()
        .zip(summary.values.iter())
        .map(|(name, value)| (name, argument(name, value)))
        .collect();

    text_with_in(
        language,
        &format!("change-{}", summary.code.as_str()),
        &args,
    )
    .into_owned()
}

fn argument<'a>(name: &str, value: &'a str) -> Arg<'a> {
    match (name, value.parse::<i64>()) {
        ("count", Ok(count)) => Arg::Count(count),
        _ => Arg::Text(value),
    }
}
