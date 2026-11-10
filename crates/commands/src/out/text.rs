//! The words of an error, as text, as a JSON line or as a kept object.

use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::output::{Paint, json_line};

use super::Json;

/// One error as a read's JSON gives it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Said {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
    /// The program's own words.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl Said {
    /// The error of `text`, a second line its hint, with the program's own
    /// words in `detail`.
    pub fn new(text: &str, detail: Option<&str>) -> Self {
        let (message, hint) = text
            .split_once('\n')
            .map_or((text, None), |(message, hint)| (message, Some(hint)));

        Self {
            message: message.to_owned(),
            hint: hint.map(str::to_owned),
            detail: detail
                .filter(|detail| !detail.trim().is_empty())
                .map(str::to_owned),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sink {
    /// A change's JSON Lines on standard output.
    Line,
    /// `error:` and `hint:` lines on standard error.
    Text,
    /// Kept for a read's JSON object on standard error.
    Kept,
}

pub fn said(paint: Paint, text: &str, json: bool) -> String {
    if json {
        return crate::apply::error_line(text, None);
    }

    match text.split_once('\n') {
        Some((error, hint)) => format!("{}\n{}\n", paint.error(error), paint.hint(hint)),
        None => format!("{}\n", paint.error(text)),
    }
}

/// Where an error goes.
pub const fn sink(json: Json) -> Sink {
    match json {
        Json::Lines => Sink::Line,
        Json::Read => Sink::Kept,
        Json::Off => Sink::Text,
    }
}

pub const fn lost_sink(json: Json) -> Sink {
    match json {
        Json::Lines | Json::Read => Sink::Kept,
        Json::Off => Sink::Text,
    }
}

/// `{"schema": 1, "error": {...}}` with the exit code as `code` and its
/// name as `reason`.
pub fn error_object(said: &Said, exit: Exit) -> String {
    #[derive(Serialize)]
    struct Coded<'a> {
        #[serde(flatten)]
        said: &'a Said,
        code: u8,
        reason: &'static str,
    }

    #[derive(Serialize)]
    struct Object<'a> {
        error: Coded<'a>,
    }

    json_line(&Object {
        error: Coded {
            said,
            code: exit.code(),
            reason: exit.reason(),
        },
    })
}

pub fn failed_text(paint: Paint, what: &str, raw: &str, verbose: bool, json: bool) -> String {
    if json {
        return crate::apply::error_line(what, Some(raw));
    }

    failure_text(paint, what, raw, verbose)
}

pub fn failure_text(paint: Paint, what: &str, raw: &str, verbose: bool) -> String {
    let mut text = format!("{}\n", paint.error(what));
    if raw.trim().is_empty() {
        return text;
    }

    if verbose {
        text.push_str(&paint.detail(raw));
    } else {
        let hint = paint.hint("add --verbose to see the program's own words");
        if !hint.is_empty() {
            text.push_str(&hint);
            text.push('\n');
        }
    }

    text
}
