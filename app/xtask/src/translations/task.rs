use std::fs;
use std::path::PathBuf;

use serde_json::Value as Json;

use super::{
    Files, folder_for, import_locale, is_locale, locale_json, pretty, read_files, source_json,
    tx_code,
};
use crate::{ENGLISH, Failure, LOCALES_DIR, flags, root};
const JSON_DIR: &str = "i18n";
const JSON_PREFIX: &str = "garuda-toolbox_";
pub(super) const SOURCE_CODE: &str = "en";

fn problem(line: String) -> Failure {
    Failure::Problems(vec![line])
}

fn folders() -> Result<Vec<String>, Failure> {
    let mut found: Vec<String> = fs::read_dir(root().join(LOCALES_DIR))
        .map_err(|error| problem(format!("{LOCALES_DIR}: {error}")))?
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| is_locale(name))
        .collect();

    found.sort();
    Ok(found)
}

fn locale_files(folder: &str) -> Result<Files, Failure> {
    read_files(&root().join(LOCALES_DIR).join(folder)).map_err(problem)
}

pub(super) fn json_path(code: &str) -> PathBuf {
    root()
        .join(JSON_DIR)
        .join(format!("{JSON_PREFIX}{code}.json"))
}

/// `cargo xtask i18n-export [--translations] [--check]`: writes the
/// source JSON, with `--translations` also each locale's current
/// translations (to seed Transifex). `--check` fails when the source
/// JSON is out of date.
pub fn i18n_export(args: &[String]) -> Result<String, Failure> {
    let given = flags(args, &["--check", "--translations"]).ok_or(Failure::Usage)?;
    let check = given.contains(&"--check");

    let folders = folders()?;
    let english = locale_files(ENGLISH)?;
    let (source, untranslatable) = source_json(&english).map_err(problem)?;
    let mut outputs = vec![(json_path(SOURCE_CODE), pretty(&source))];

    if given.contains(&"--translations") {
        for folder in folders.iter().filter(|folder| *folder != ENGLISH) {
            let json = locale_json(&english, &locale_files(folder)?).map_err(problem)?;
            outputs.push((json_path(&tx_code(folder, &folders)), pretty(&json)));
        }
    }

    let mut stale = Vec::new();
    for (path, text) in &outputs {
        if fs::read_to_string(path).ok().as_deref() == Some(text.as_str()) {
            continue;
        }

        if check {
            stale.push(format!(
                "{} is out of date: run `cargo xtask i18n-export`",
                path.display()
            ));
        } else {
            fs::create_dir_all(root().join(JSON_DIR)).ok();
            fs::write(path, text)
                .map_err(|error| problem(format!("{}: {error}", path.display())))?;
        }
    }

    if !stale.is_empty() {
        return Err(Failure::Problems(stale));
    }

    let mut report = format!("{} JSON files up to date", outputs.len());
    for line in untranslatable {
        report.push_str("\nnot for Transifex: ");
        report.push_str(&line);
    }

    Ok(report)
}

fn codes() -> Vec<String> {
    let Ok(entries) = fs::read_dir(root().join(JSON_DIR)) else {
        return Vec::new();
    };

    let mut found: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter_map(|name| {
            Some(
                name.strip_prefix(JSON_PREFIX)?
                    .strip_suffix(".json")?
                    .to_owned(),
            )
        })
        .filter(|code| code != SOURCE_CODE)
        .collect();

    found.sort();
    found
}

/// `cargo xtask i18n-import [--check] [--dry-run]`: writes the
/// translations of every `i18n/garuda-toolbox_<code>.json` into its
/// locale's Fluent files. English is never touched. Refused translations
/// fail the task after the others are written.
pub fn i18n_import(args: &[String]) -> Result<String, Failure> {
    let given = flags(args, &["--check", "--dry-run"]).ok_or(Failure::Usage)?;
    let check = given.contains(&"--check");
    let dry_run = given.contains(&"--dry-run");

    let folders = folders()?;
    let english = locale_files(ENGLISH)?;
    let mut changed_paths = Vec::new();
    let mut problems = Vec::new();

    for code in codes() {
        let folder = folder_for(&code, &folders);
        let path = json_path(&code);
        let text = fs::read_to_string(&path)
            .map_err(|error| problem(format!("{}: {error}", path.display())))?;

        let json: Json = serde_json::from_str(&text)
            .map_err(|error| problem(format!("{}: {error}", path.display())))?;

        let (changed, refused) = import_locale(&english, &locale_files(&folder)?, &json);
        problems.extend(refused.into_iter().map(|line| format!("{folder}/{line}")));

        for (stem, new) in changed {
            let shown = format!("{LOCALES_DIR}/{folder}/{stem}.ftl");
            let target = root().join(&shown);

            if !check && !dry_run {
                fs::create_dir_all(root().join(LOCALES_DIR).join(&folder)).ok();
                fs::write(&target, new).map_err(|error| problem(format!("{shown}: {error}")))?;
            }

            changed_paths.push(shown);
        }
    }

    if check {
        problems.extend(changed_paths.iter().map(|path| {
            format!("{path} is behind its translations: run `cargo xtask i18n-import`")
        }));
    }

    if !problems.is_empty() {
        return Err(Failure::Problems(problems));
    }

    let mut report = format!(
        "{} {} Fluent files",
        if dry_run { "would change" } else { "changed" },
        changed_paths.len()
    );

    for path in changed_paths {
        report.push_str("\n  ");
        report.push_str(&path);
    }

    Ok(report)
}
