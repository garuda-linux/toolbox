//! The app's own settings: `settings.toml`, shared with the window, and
//! the autostart link. Theme and motion live in the window's own state
//! and are only changed there.

#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::path::PathBuf;

use clap::ValueEnum as _;
use serde::Serialize;
use toolbox_catalog::{ColourSchemeCatalog, SchemeMode};
use toolbox_cli::Exit;
use toolbox_cli::args::{Read, SettingKey, SettingsCommand};
use toolbox_cli::output::{Row, json};
use toolbox_config::{
    Autostart, DESKTOP_FILE, InvalidLanguage, Language, LogLevel, MAX_MINUTES, MIN_MINUTES, Paths,
    RefreshInterval, Settings,
};
use toolbox_plan::SchemeId;
use toolbox_services::catalogs::catalog_dir;

use super::out::Listing;
use crate::out::Out;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SettingRow {
    #[serde(rename = "id")]
    pub key: &'static str,
    pub value: String,
}

/// Why `set` refused a value.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SettingError {
    #[error(transparent)]
    Language(#[from] InvalidLanguage),
    #[error("`{0}` is no number of minutes, {MIN_MINUTES} to {MAX_MINUTES}, or never")]
    Minutes(String),
    /// The words of `on_off` and `log_level`, which stay `String` errors
    /// as clap value parsers have them.
    #[error("{0}")]
    Word(String),
    #[error("`{value}` is none of {offered}")]
    Scheme { value: String, offered: String },
    #[error("start at login is no setting of the file")]
    NotInFile,
}

pub fn command(command: SettingsCommand, out: &mut Out<'_>) -> Exit {
    let paths = Paths::from_process();
    let file = paths.settings_file();
    let autostart = Autostart::new(paths.autostart_dir(), PathBuf::from(DESKTOP_FILE));
    let loaded = Settings::load(&file);
    if let Some(problem) = &loaded.problem {
        let text = format!("the settings file is unusable, the defaults are shown: {problem}");
        writeln!(out, "{}", out.paint().warning(&text)).ok();
    }

    let settings = loaded.settings;
    match command {
        SettingsCommand::List(list) => {
            let rows = rows(&settings, autostart.is_enabled());
            super::filter::print_all(&list, &rows, out, setting_listing)
        }
        SettingsCommand::Get(args) => {
            let text = get_text(args.read, &settings, args.key, autostart.is_enabled());
            write!(out, "{text}").ok();
            Exit::Success
        }
        SettingsCommand::Set(args) if args.key == SettingKey::StartAtLogin => {
            let switched = match on_off(&args.value) {
                Ok(true) => autostart.enable(),
                Ok(false) => autostart.disable(),
                Err(message) => return usage(&message, out),
            };

            switched.map_or_else(
                |error| {
                    let text = format!("the autostart link was not changed: {error}");
                    crate::say_error(out, &text);
                    Exit::Failure
                },
                |()| Exit::Success,
            )
        }
        SettingsCommand::Set(args) => match set(settings, args.key, &args.value, &schemes()) {
            Ok(changed) => changed.save(&file).map_or_else(
                |error| {
                    crate::say_error(out, &format!("the settings were not saved: {error}"));
                    Exit::Failure
                },
                |()| Exit::Success,
            ),
            Err(error) => usage(&error.to_string(), out),
        },
    }
}

/// `settings` with `key` set to `value`. Start at login is no part of the
/// file and is refused here.
///
/// # Errors
///
/// [`SettingError`], what `value` should have been.
pub fn set(
    mut settings: Settings,
    key: SettingKey,
    value: &str,
    schemes: &ColourSchemeCatalog,
) -> Result<Settings, SettingError> {
    let switch = |word: &str| on_off(word).map_err(SettingError::Word);
    match key {
        SettingKey::Language if value == "system" => settings.general.language = None,
        SettingKey::Language => settings.general.language = Some(Language::new(value)?),
        SettingKey::UpdateCheck if value == "never" => {
            settings.status.refresh = RefreshInterval::Off;
        }
        SettingKey::UpdateCheck => {
            let minutes: u32 = value
                .parse()
                .map_err(|_nan| SettingError::Minutes(value.to_owned()))?;

            settings.status.refresh = RefreshInterval::clamped(minutes);
        }
        SettingKey::UserServices => settings.services.show_user_units = switch(value)?,
        SettingKey::CopyReports => settings.diagnostics.copy_to_clipboard = switch(value)?,
        SettingKey::Aur => settings.packages.show_aur = switch(value)?,
        SettingKey::LogLevel => {
            settings.general.log_level = log_level(value).map_err(SettingError::Word)?;
        }
        SettingKey::DarkScheme => {
            settings.colours.dark_scheme = scheme(value, SchemeMode::Dark, schemes)?;
        }
        SettingKey::LightScheme => {
            settings.colours.light_scheme = scheme(value, SchemeMode::Light, schemes)?;
        }
        SettingKey::StartAtLogin => return Err(SettingError::NotInFile),
    }

    Ok(settings)
}

pub fn value(settings: &Settings, key: SettingKey, autostart: bool) -> String {
    let switch = |on: bool| on_off_text(on).to_owned();
    match key {
        SettingKey::Language => settings
            .general
            .language
            .as_ref()
            .map_or_else(|| "system".to_owned(), ToString::to_string),
        SettingKey::StartAtLogin => switch(autostart),
        SettingKey::UpdateCheck => match settings.status.refresh {
            RefreshInterval::Off => "never".to_owned(),
            RefreshInterval::Minutes(minutes) => minutes.to_string(),
        },
        SettingKey::UserServices => switch(settings.services.show_user_units),
        SettingKey::CopyReports => switch(settings.diagnostics.copy_to_clipboard),
        SettingKey::Aur => switch(settings.packages.show_aur),
        SettingKey::LogLevel => log_level_name(settings.general.log_level).to_owned(),
        SettingKey::DarkScheme => scheme_name(settings.colours.dark_scheme.as_ref()),
        SettingKey::LightScheme => scheme_name(settings.colours.light_scheme.as_ref()),
    }
}

pub fn rows(settings: &Settings, autostart: bool) -> Vec<SettingRow> {
    SettingKey::value_variants()
        .iter()
        .map(|&key| SettingRow {
            key: key_name(key),
            value: value(settings, key, autostart),
        })
        .collect()
}

pub fn get_text(read: Read, settings: &Settings, key: SettingKey, autostart: bool) -> String {
    let value = value(settings, key, autostart);
    if read.json {
        json(&SettingRow {
            key: key_name(key),
            value,
        })
    } else {
        format!("{value}\n")
    }
}

/// The settings, the columns plain enough to need no names. What a setting
/// takes is its help text from `settings set --help`.
pub fn setting_listing(rows: &[SettingRow]) -> Listing<'_> {
    Listing {
        title: "Settings",
        summary: Some(rows.len().to_string()),
        head: &[],
        rows: rows
            .iter()
            .map(|row| Row::plain(vec![row.key.to_owned(), row.value.clone(), about(row.key)]))
            .collect(),
        hint: Some("garuda-toolbox settings set <setting> <value>".to_owned()),
    }
}

fn about(key: &str) -> String {
    SettingKey::from_str(key, false)
        .ok()
        .and_then(|key| key.to_possible_value())
        .and_then(|value| value.get_help().map(ToString::to_string))
        .unwrap_or_default()
}

const fn key_name(key: SettingKey) -> &'static str {
    match key {
        SettingKey::Language => "language",
        SettingKey::StartAtLogin => "start-at-login",
        SettingKey::UpdateCheck => "update-check",
        SettingKey::UserServices => "user-services",
        SettingKey::CopyReports => "copy-reports",
        SettingKey::Aur => "aur",
        SettingKey::LogLevel => "log-level",
        SettingKey::DarkScheme => "dark-scheme",
        SettingKey::LightScheme => "light-scheme",
    }
}

pub(super) fn on_off(value: &str) -> Result<bool, String> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => Err(format!("`{value}` is neither on nor off")),
    }
}

pub(super) const fn on_off_text(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

const fn log_level_name(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    }
}

fn log_level(value: &str) -> Result<LogLevel, String> {
    Ok(match value {
        "error" => LogLevel::Error,
        "warn" => LogLevel::Warn,
        "info" => LogLevel::Info,
        "debug" => LogLevel::Debug,
        "trace" => LogLevel::Trace,
        _ => {
            return Err(format!(
                "`{value}` is not error, warn, info, debug or trace"
            ));
        }
    })
}

fn schemes() -> ColourSchemeCatalog {
    let dir = catalog_dir(|key| std::env::var_os(key));
    ColourSchemeCatalog::load(&dir).unwrap_or_else(|error| {
        tracing::warn!("the colour schemes are unavailable: {error}");
        ColourSchemeCatalog::default()
    })
}

/// `value` as the scheme of `mode`, `None` for `system`.
fn scheme(
    value: &str,
    mode: SchemeMode,
    schemes: &ColourSchemeCatalog,
) -> Result<Option<SchemeId>, SettingError> {
    if value == "system" {
        return Ok(None);
    }

    let found = SchemeId::new(value)
        .ok()
        .filter(|id| schemes.find(id, mode).is_some());

    found.map(Some).ok_or_else(|| {
        let offered: Vec<&str> = std::iter::once("system")
            .chain(schemes.of(mode).map(|scheme| scheme.id().as_str()))
            .collect();

        SettingError::Scheme {
            value: value.to_owned(),
            offered: offered.join(", "),
        }
    })
}

fn scheme_name(id: Option<&SchemeId>) -> String {
    id.map_or_else(|| "system".to_owned(), ToString::to_string)
}

fn usage(message: &str, out: &mut Out<'_>) -> Exit {
    crate::say_error(out, message);
    Exit::Usage
}
