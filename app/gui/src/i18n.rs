//! The window's side of the texts of `toolbox-services`. The language in
//! use sits on the egui context, so every page reaches it.

pub mod changes;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use egui::{Context, Id};
use fluent_templates::LanguageIdentifier;
use toolbox_config::Language;
pub use toolbox_services::i18n::{
    Arg, FALLBACK, available, language_from_env, name_of, text_in, text_or_in, text_with_in,
};

/// Makes `language` the one every following [`text`] call uses.
pub fn set_language(ctx: &Context, language: &LanguageIdentifier) {
    ctx.data_mut(|data| data.insert_temp(language_id(), language.clone()));
}

pub fn text(ctx: &Context, id: &str) -> Cow<'static, str> {
    text_in(language(ctx), id)
}

/// [`text`], or `fallback` for an id no file has: catalog texts, whose
/// English stands in the catalog itself.
pub fn text_or(ctx: &Context, id: &str, fallback: &str) -> Cow<'static, str> {
    text_or_in(&language(ctx), id, fallback)
}

pub fn text_with(ctx: &Context, id: &str, args: &[(&'static str, Arg<'_>)]) -> Cow<'static, str> {
    text_with_in(&language(ctx), id, args)
}

pub fn language(ctx: &Context) -> LanguageIdentifier {
    ctx.data(|data| data.get_temp::<LanguageIdentifier>(language_id()))
        .unwrap_or(FALLBACK)
}

fn language_id() -> Id {
    Id::unique("toolbox_gui::i18n::language")
}

/// The language the window shows: the one the settings choose when there
/// are texts for it, else the `system` one.
pub fn shown(chosen: Option<&Language>, system: &LanguageIdentifier) -> LanguageIdentifier {
    chosen
        .and_then(|code| code.as_str().parse::<LanguageIdentifier>().ok())
        .filter(|language| available().contains(language))
        .unwrap_or_else(|| system.clone())
}
