//! The sentences of the values a catalog problem wraps: a name, a home or
//! config path, a config word or text, and the check of a duplicate. Each
//! variant has its own id, so the window words it as its own message does.

use egui::Context;
use toolbox_catalog::Check;
use toolbox_plan::{
    InvalidConfigFilePath, InvalidConfigText, InvalidConfigWord, InvalidHomePath, InvalidName,
};

use super::{counted_text, escaped_text, plain_text};
use crate::i18n::{self, Arg};
use crate::probe_texts;

/// A name the workspace's rules refuse, worded as the probe texts word one.
pub(super) fn name(ctx: &Context, invalid: &InvalidName) -> String {
    let kind = i18n::text(ctx, probe_texts::kind_id(invalid.kind())).into_owned();
    let reason = probe_texts::reason_words(ctx, invalid.reason());
    let value = invalid.shown_value();
    let args = [
        ("kind", Arg::from(&kind)),
        ("value", Arg::from(&value)),
        ("reason", Arg::from(&reason)),
    ];

    i18n::text_with(ctx, "catalog-problem-bad-name", &args).into_owned()
}

/// The check another entry watches, with its kind of check in words.
pub(super) fn duplicate_check(ctx: &Context, check: &Check) -> String {
    let watched = match check {
        Check::Package(package) => named(ctx, "catalog-check-package", package.as_str()),
        Check::Service(unit) => named(ctx, "catalog-check-service", unit.as_str()),
        Check::UserService(unit) => named(ctx, "catalog-check-user-service", unit.as_str()),
        Check::Group(group) => named(ctx, "catalog-check-group", group.as_str()),
        Check::UserConfig(edit) => {
            let text = edit.to_string();
            let args = [("edit", Arg::from(&text))];

            i18n::text_with(ctx, "catalog-check-user-config", &args).into_owned()
        }
    };

    escaped_text(ctx, "catalog-problem-duplicate-check", &watched)
}

/// A path below the home that breaks a rule of its type.
pub(super) fn home_path(ctx: &Context, error: &InvalidHomePath) -> String {
    match error {
        InvalidHomePath::Empty => plain_text(ctx, "catalog-problem-home-path-empty"),
        InvalidHomePath::TooLong { max } => {
            counted_text(ctx, "catalog-problem-home-path-too-long", *max)
        }
        InvalidHomePath::Absolute(path) => {
            escaped_text(ctx, "catalog-problem-home-path-absolute", path)
        }
        InvalidHomePath::InvalidCharacter(character) => escaped_text(
            ctx,
            "catalog-problem-home-path-invalid-character",
            &character.to_string(),
        ),
        InvalidHomePath::InvalidComponent(path) => {
            escaped_text(ctx, "catalog-problem-home-path-invalid-component", path)
        }
    }
}

/// A config word that breaks a rule of its type.
pub(super) fn config_word(ctx: &Context, error: &InvalidConfigWord) -> String {
    match error {
        InvalidConfigWord::Empty => plain_text(ctx, "catalog-problem-config-word-empty"),
        InvalidConfigWord::TooLong { max } => {
            counted_text(ctx, "catalog-problem-config-word-too-long", *max)
        }
        InvalidConfigWord::InvalidCharacter(character) => escaped_text(
            ctx,
            "catalog-problem-config-word-invalid-character",
            &character.to_string(),
        ),
        InvalidConfigWord::Padded(word) => {
            escaped_text(ctx, "catalog-problem-config-word-padded", word)
        }
    }
}

/// A config file text the helper refuses to write.
pub(super) fn config_text(ctx: &Context, error: &InvalidConfigText) -> String {
    match error {
        InvalidConfigText::TooLarge { max } => {
            counted_text(ctx, "catalog-problem-config-text-too-large", *max)
        }
        InvalidConfigText::NotText => plain_text(ctx, "catalog-problem-config-text-not-text"),
    }
}

/// A root-owned config file path that breaks a rule of its type.
pub(super) fn config_path(ctx: &Context, error: &InvalidConfigFilePath) -> String {
    match error {
        InvalidConfigFilePath::Empty => plain_text(ctx, "catalog-problem-config-path-empty"),
        InvalidConfigFilePath::TooLong { max } => {
            counted_text(ctx, "catalog-problem-config-path-too-long", *max)
        }
        InvalidConfigFilePath::Relative(path) => {
            escaped_text(ctx, "catalog-problem-config-path-relative", path)
        }
        InvalidConfigFilePath::InvalidCharacter(character) => escaped_text(
            ctx,
            "catalog-problem-config-path-invalid-character",
            &character.to_string(),
        ),
        InvalidConfigFilePath::InvalidComponent(path) => {
            escaped_text(ctx, "catalog-problem-config-path-invalid-component", path)
        }
        InvalidConfigFilePath::OutsideRoots(path) => {
            escaped_text(ctx, "catalog-problem-config-path-outside-roots", path)
        }
        InvalidConfigFilePath::Withheld(path) => {
            escaped_text(ctx, "catalog-problem-config-path-withheld", path)
        }
    }
}

fn named(ctx: &Context, id: &str, name: &str) -> String {
    let args = [("name", Arg::from(name))];

    i18n::text_with(ctx, id, &args).into_owned()
}
