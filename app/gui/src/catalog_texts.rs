//! The window's words for a catalog that failed to load. Each [`Problem`]
//! picks its sentence by a literal id here, its values go in as arguments,
//! and the text a TOML parser or the system printed stays data.

#[cfg(test)]
mod tests;

mod nested;

use egui::Context;
use toolbox_catalog::{FileProblem, Problem};

use crate::i18n::{self, Arg};

/// The sentence of a file that failed to load, with its problem in it.
pub fn file(ctx: &Context, failed: &FileProblem) -> String {
    match failed {
        FileProblem::Read { path, error } => {
            let path_text = path.display().to_string();
            let args = [("path", Arg::from(&path_text)), ("error", Arg::from(error))];

            i18n::text_with(ctx, "catalog-error-read", &args).into_owned()
        }
        FileProblem::Syntax { source_name, error } => {
            let args = [
                ("source", Arg::from(source_name)),
                ("error", Arg::from(error)),
            ];

            i18n::text_with(ctx, "catalog-error-syntax", &args).into_owned()
        }
        FileProblem::Invalid {
            source_name,
            id,
            problem,
        } => {
            let id_text = escape(id);
            let sentence = problem_text(ctx, problem);
            let args = [
                ("source", Arg::from(source_name)),
                ("id", Arg::from(&id_text)),
                ("problem", Arg::from(&sentence)),
            ];

            i18n::text_with(ctx, "catalog-error-entry", &args).into_owned()
        }
    }
}

/// The sentence of a problem of the catalog, its values as arguments.
pub fn problem_text(ctx: &Context, problem: &Problem) -> String {
    let plain = |id: &str| plain_text(ctx, id);
    let escaped = |id: &str, value: &str| escaped_text(ctx, id, value);
    let counted = |id: &str, value: usize| counted_text(ctx, id, value);

    match problem {
        Problem::BadId => plain("catalog-problem-bad-id"),
        Problem::DuplicateId => plain("catalog-problem-duplicate-id"),
        Problem::DuplicateCheck(check) => nested::duplicate_check(ctx, check),
        Problem::DuplicatePath(path) => escaped("catalog-problem-duplicate-path", path),
        Problem::AdminGroup(group) => escaped("catalog-problem-admin-group", group),
        Problem::EmptyName => plain("catalog-problem-empty-name"),
        Problem::EmptyTitle => plain("catalog-problem-empty-title"),
        Problem::EmptyDescription => plain("catalog-problem-empty-description"),
        Problem::HiddenCharacter(text) => escaped("catalog-problem-hidden-character", text),
        Problem::EmptyIcon => plain("catalog-problem-empty-icon"),
        Problem::NoEntries => plain("catalog-problem-no-entries"),
        Problem::BadName(name) => nested::name(ctx, name),
        Problem::EmptyRequirement => plain("catalog-problem-empty-requirement"),
        Problem::UnknownRequirement(id) => escaped("catalog-problem-unknown-requirement", id),
        Problem::RequirementRepeats(id) => escaped("catalog-problem-requirement-repeats", id),
        Problem::RequiresItself => plain("catalog-problem-requires-itself"),
        Problem::RequirementCycle(ids) => escaped("catalog-problem-requirement-cycle", &cycle(ids)),
        Problem::NoFiles => plain("catalog-problem-no-files"),
        Problem::OutsideSkel(file) => escaped("catalog-problem-outside-skel", file),
        Problem::ParentComponent(file) => escaped("catalog-problem-parent-component", file),
        Problem::BadHomePath(error) => nested::home_path(ctx, error),
        Problem::BadConfigPath(error) => nested::config_path(ctx, error),
        Problem::NotWritable(path) => escaped("catalog-problem-not-writable", path),
        Problem::ReadOnlyWritable(path) => escaped("catalog-problem-read-only-writable", path),
        Problem::ReadOnlyHome => plain("catalog-problem-read-only-home"),
        Problem::BadConfigWord(error) => nested::config_word(ctx, error),
        Problem::BadConfigText(error) => nested::config_text(ctx, error),
        Problem::EmptyContent => plain("catalog-problem-empty-content"),
        Problem::ReloadWithoutUserConfig => plain("catalog-problem-reload-without-user-config"),
        Problem::UnknownReload(name) => escaped("catalog-problem-unknown-reload", name),
        Problem::RestartWithoutUserConfig => plain("catalog-problem-restart-without-user-config"),
        Problem::RestartRepeats(unit) => escaped("catalog-problem-restart-repeats", unit),
        Problem::EmptyDesktop => plain("catalog-problem-empty-desktop"),
        Problem::AlsoWithoutPackage => plain("catalog-problem-also-without-package"),
        Problem::EmptyAlso => plain("catalog-problem-empty-also"),
        Problem::AlsoRepeats(name) => escaped("catalog-problem-also-repeats", name),
        Problem::EmptyConflicts => plain("catalog-problem-empty-conflicts"),
        Problem::ConflictRepeats(id) => escaped("catalog-problem-conflict-repeats", id),
        Problem::UnknownConflict(id) => escaped("catalog-problem-unknown-conflict", id),
        Problem::ConflictsItself => plain("catalog-problem-conflicts-itself"),
        Problem::ConflictsRequirement(id) => escaped("catalog-problem-conflicts-requirement", id),
        Problem::BadIcon(icon) => escaped("catalog-problem-bad-icon", icon),
        Problem::MissingIcon(icon) => escaped("catalog-problem-missing-icon", icon),
        Problem::BadGlyph(glyph) => escaped("catalog-problem-bad-glyph", glyph),
        Problem::ProtectedAlso(package) => escaped("catalog-problem-protected-also", package),
        Problem::BadUrl(url) => escaped("catalog-problem-bad-url", url),
        Problem::BadColour(colour) => escaped("catalog-problem-bad-colour", &colour.0),
        Problem::AnsiCount(count) => counted("catalog-problem-ansi-count", *count),
        Problem::EmptyLicence => plain("catalog-problem-empty-licence"),
    }
}

/// The entries of a cycle, closed back on the first one.
fn cycle(ids: &[String]) -> String {
    let first = ids.first().map_or("", String::as_str);

    format!("{} -> {first}", ids.join(" -> "))
}

fn plain_text(ctx: &Context, id: &str) -> String {
    i18n::text(ctx, id).into_owned()
}

fn escaped_text(ctx: &Context, id: &str, value: &str) -> String {
    let escaped = escape(value);
    let args = [("value", Arg::from(&escaped))];

    i18n::text_with(ctx, id, &args).into_owned()
}

fn counted_text(ctx: &Context, id: &str, value: usize) -> String {
    let args = [("value", Arg::from(value))];

    i18n::text_with(ctx, id, &args).into_owned()
}

/// The value with its control characters and bidi overrides spelled out,
/// so the sentence shows what the file holds rather than hiding it.
fn escape(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}
