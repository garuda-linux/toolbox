//! The window's side of the texts of `toolbox-services`. The language in
//! use sits on the egui context, so every page reaches it.

pub mod changes;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use egui::{Context, Id};
use fluent_templates::LanguageIdentifier;
pub use toolbox_services::i18n::{
    FALLBACK, available, language_from_env, name_of, text_in, text_or_in, text_with_in,
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

pub fn text_with(ctx: &Context, id: &str, args: &[(&'static str, &str)]) -> Cow<'static, str> {
    text_with_in(&language(ctx), id, args)
}

pub fn language(ctx: &Context) -> LanguageIdentifier {
    ctx.data(|data| data.get_temp::<LanguageIdentifier>(language_id()))
        .unwrap_or(FALLBACK)
}

fn language_id() -> Id {
    Id::unique("toolbox_gui::i18n::language")
}
