//! The app's Fluent texts for Transifex, which reads no Fluent: one
//! Structured JSON file per language with ICU messages, and back. The
//! source is `i18n/garuda-toolbox_en.json`, the translations are
//! `i18n/garuda-toolbox_<code>.json` in Transifex's language codes.

mod ftl;
mod icu;
mod model;
mod task;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ftl::Edit;
pub use ftl::{plain_value, unmarked_values};
use model::{Choice, Message, Part, Plural, Value, messages};
use serde_json::{Map, Value as Json};
pub use task::{i18n_export, i18n_import};

/// A locale's Fluent files by name without `.ftl`.
type Files = BTreeMap<String, String>;

/// The messages of `stem.ftl`, or why the file is no valid Fluent.
fn parsed(stem: &str, text: &str) -> Result<Vec<Message>, String> {
    messages(text).map_err(|error| format!("{stem}.ftl: {error}"))
}

fn read_files(dir: &Path) -> Result<Files, String> {
    let mut files = Files::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(files);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "ftl")
            && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
        {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;

            files.insert(stem.to_owned(), text);
        }
    }

    Ok(files)
}

/// A language tag as the app's locale folders use them: a language of
/// two or three small letters, then subtags joined with `-`. Anything
/// else in the folder (tool leftovers) is no locale.
fn is_locale(name: &str) -> bool {
    let mut subtags = name.split('-');
    let language = subtags.next().unwrap_or_default();

    (2..=3).contains(&language.len())
        && language.chars().all(|letter| letter.is_ascii_lowercase())
        && subtags.all(|subtag| {
            !subtag.is_empty() && subtag.chars().all(|letter| letter.is_ascii_alphanumeric())
        })
}

/// The language subtag, or the whole tag with `_` when two folders share
/// the language.
fn tx_code(folder: &str, folders: &[String]) -> String {
    let language = folder.split('-').next().unwrap_or(folder);
    let shared = folders
        .iter()
        .filter(|other| other.split('-').next() == Some(language))
        .count();

    if shared > 1 {
        folder.replace('-', "_")
    } else {
        language.to_owned()
    }
}

fn folder_for(code: &str, folders: &[String]) -> String {
    let tag = code.replace('_', "-");
    if folders.contains(&tag) {
        return tag;
    }

    let mut same = folders
        .iter()
        .filter(|folder| folder.split('-').next() == Some(tag.as_str()));

    match (same.next(), same.next()) {
        (Some(folder), None) => folder.clone(),
        (Some(_), Some(_)) | (None, _) => tag,
    }
}

fn entry(icu: String, note: Option<&str>) -> Json {
    let mut object = Map::new();
    object.insert("string".to_owned(), Json::String(icu));
    if let Some(note) = note {
        object.insert(
            "developer_comment".to_owned(),
            Json::String(note.to_owned()),
        );
    }

    Json::Object(object)
}

fn to_json(value: &Value, note: Option<&str>) -> Json {
    match value {
        Value::Plain(parts) => entry(icu::icu(parts), note),
        Value::Choice(Choice {
            var,
            default: _,
            variants,
        }) => Json::Object(
            variants
                .iter()
                .map(|(key, parts)| {
                    let used = format!("Used when `{var}` is `{key}`.");
                    let note = match note {
                        Some(note) => format!("{note}\n{used}"),
                        None => used,
                    };

                    (key.clone(), entry(icu::icu(parts), Some(&note)))
                })
                .collect(),
        ),
    }
}

/// The source JSON and every message that cannot go to Transifex.
///
/// # Errors
///
/// The first English file that is no valid Fluent.
fn source_json(english: &Files) -> Result<(Json, Vec<String>), String> {
    let mut root = Map::new();
    let mut untranslatable = Vec::new();

    for (stem, text) in english {
        let mut file = Map::new();

        for message in parsed(stem, text)? {
            match &message.value {
                Ok(value) => {
                    file.insert(message.id.clone(), to_json(value, message.note.as_deref()));
                }
                Err(reason) => untranslatable.push(format!("{stem}.ftl: {} {reason}", message.id)),
            }
        }

        root.insert(stem.clone(), Json::Object(file));
    }

    Ok((Json::Object(root), untranslatable))
}

/// The translations a locale has today, in the source's shape. Messages
/// still marked untranslated stay out.
///
/// # Errors
///
/// The first English or locale file that is no valid Fluent.
fn locale_json(english: &Files, locale: &Files) -> Result<Json, String> {
    let mut root = Map::new();

    for (stem, english_text) in english {
        let Some(text) = locale.get(stem) else {
            continue;
        };

        let marked = ftl::marked(text);
        let shapes: BTreeMap<String, Value> = parsed(stem, english_text)?
            .into_iter()
            .filter_map(|message| Some((message.id, message.value.ok()?)))
            .collect();

        let mut file = Map::new();

        for message in parsed(stem, text)? {
            let (Some(source), Ok(value)) = (shapes.get(&message.id), &message.value) else {
                continue;
            };

            if !marked.contains(&message.id) && same_shape(source, value) {
                file.insert(message.id.clone(), to_json(value, None));
            }
        }

        if !file.is_empty() {
            root.insert(stem.clone(), Json::Object(file));
        }
    }

    Ok(Json::Object(root))
}

fn same_shape(source: &Value, value: &Value) -> bool {
    match (source, value) {
        (Value::Plain(_), Value::Plain(_)) => true,
        (Value::Choice(english), Value::Choice(translated)) => english.var == translated.var,
        (Value::Plain(_), Value::Choice(_)) | (Value::Choice(_), Value::Plain(_)) => false,
    }
}

/// What `json` brings for one locale: the changed files and every
/// translation that was refused. A file that is no valid Fluent is left
/// as it is and reported, so no message is written twice.
fn import_locale(english: &Files, locale: &Files, json: &Json) -> (Files, Vec<String>) {
    let mut changed = Files::new();
    let mut problems = Vec::new();

    for (stem, english_text) in english {
        let Some(incoming) = json.get(stem) else {
            continue;
        };

        let current = locale.get(stem).cloned().unwrap_or_default();
        let (text, refused) = match import_file(stem, english_text, &current, incoming) {
            Ok(imported) => imported,
            Err(error) => {
                problems.push(error);
                continue;
            }
        };

        problems.extend(
            refused
                .into_iter()
                .map(|line| format!("{stem}.ftl: {line}")),
        );

        if text != current {
            changed.insert(stem.clone(), text);
        }
    }

    (changed, problems)
}

/// One locale file with `incoming` written in. Only a message whose
/// translation changed is rewritten, the rest keeps its bytes.
///
/// # Errors
///
/// The English or the locale file is no valid Fluent.
fn import_file(
    stem: &str,
    english: &str,
    current: &str,
    incoming: &Json,
) -> Result<(String, Vec<String>), String> {
    let existing: BTreeMap<String, Option<Value>> = parsed(stem, current)?
        .into_iter()
        .map(|message| (message.id, message.value.ok()))
        .collect();

    let marked = ftl::marked(current);
    let mut edits = Vec::new();
    let mut refused = Vec::new();
    let mut anchor: Option<String> = None;

    for message in parsed(stem, english)? {
        let present = existing.contains_key(&message.id);
        let Ok(source) = &message.value else {
            continue;
        };

        match translation(source, incoming.get(&message.id)) {
            Some(Ok(value)) if value != *source => {
                let kept = existing.get(&message.id) == Some(&Some(value.clone()))
                    && !marked.contains(&message.id);

                let lines = ftl::render(&message.id, &value);

                match (present, kept) {
                    (true, true) => {}
                    (true, false) => edits.push(Edit::Replace {
                        id: message.id.clone(),
                        lines,
                    }),
                    (false, _) => edits.push(Edit::Insert {
                        after: anchor.clone(),
                        lines,
                    }),
                }
            }
            Some(Err(reason)) => refused.push(format!("{} {reason}", message.id)),
            Some(Ok(_)) | None => {}
        }

        if present {
            anchor = Some(message.id.clone());
        }
    }

    Ok((ftl::apply(current, &edits), refused))
}

/// The incoming translation in the source's shape, `None` when it has
/// none.
fn translation(source: &Value, incoming: Option<&Json>) -> Option<Result<Value, String>> {
    let incoming = incoming?;

    let value = match source {
        Value::Plain(_) => {
            let text = incoming.get("string")?.as_str()?;
            icu::parse(text).map(Value::Plain)
        }
        Value::Choice(Choice {
            var,
            default,
            variants,
        }) => variants
            .iter()
            .map(|(key, _)| {
                let text = incoming.get(key)?.get("string")?.as_str()?;
                Some(icu::parse(text).map(|parts| (key.clone(), parts)))
            })
            .collect::<Option<Result<Vec<_>, String>>>()?
            .map(|variants| {
                Value::Choice(Choice {
                    var: var.clone(),
                    default: default.clone(),
                    variants,
                })
            }),
    };

    Some(value.and_then(|value| {
        let allowed = variables(source);
        let foreign = variables(&value)
            .into_iter()
            .find(|name| !allowed.contains(name));

        foreign.map_or(Ok(value), |name| {
            Err(format!(
                "uses `{name}`, which the English text does not have"
            ))
        })
    }))
}

fn variables(value: &Value) -> Vec<String> {
    let mut found = Vec::new();

    match value {
        Value::Plain(parts) => collect_variables(parts, &mut found),
        Value::Choice(Choice {
            var,
            default: _,
            variants,
        }) => {
            found.push(var.clone());
            for (_, branch) in variants {
                collect_variables(branch, &mut found);
            }
        }
    }

    found
}

fn collect_variables(parts: &[Part], found: &mut Vec<String>) {
    for part in parts {
        match part {
            Part::Text(_) => {}
            Part::Var(name) => found.push(name.clone()),
            Part::Plural(Plural { var, variants }) => {
                found.push(var.clone());
                for (_, branch) in variants {
                    collect_variables(branch, found);
                }
            }
        }
    }
}

fn pretty(json: &Json) -> String {
    let mut text = serde_json::to_string_pretty(json).unwrap_or_else(|_| "{}".to_owned());
    text.push('\n');
    text
}
