use std::time::{SystemTime, UNIX_EPOCH};

use crate::calendar::Utc;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Plain,
    Color,
}

/// `text` in the ANSI colour `code` (SGR parameters) when colouring.
fn paint(style: Style, code: &str, text: &str) -> String {
    match style {
        Style::Plain => text.to_owned(),
        Style::Color => format!("\x1b[{code}m{text}\x1b[0m"),
    }
}

const fn level_color(level: log::Level) -> &'static str {
    match level {
        log::Level::Error => "1;31",
        log::Level::Warn => "33",
        log::Level::Info => "32",
        log::Level::Debug => "34",
        log::Level::Trace => "35",
    }
}

/// One record: `2026-10-04T09:30:29.123Z INFO  target: message`. Continuation
/// lines of multi-line messages are indented.
///
/// Control characters are dropped: a message that quotes program output
/// could otherwise drive the terminal of whoever reads the log. A carriage
/// return goes too, or the text after it could hide the start of its line.
pub fn format_record(
    time: SystemTime,
    level: log::Level,
    target: &str,
    message: &str,
    style: Style,
) -> String {
    let printable: String = message
        .chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\t' | '\n'))
        .collect();

    let mut lines = printable.trim_end().lines();
    let mut out = format!(
        "{} {} {}: {}\n",
        paint(style, "2", &timestamp(time)),
        paint(style, level_color(level), &format!("{level:<5}")),
        paint(style, "36", target),
        lines.next().unwrap_or_default()
    );

    for line in lines {
        out.push_str("    ");
        out.push_str(line);
        out.push('\n');
    }

    out
}

/// UTC ISO 8601 with milliseconds.
pub fn timestamp(time: SystemTime) -> String {
    let since = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let Utc {
        year,
        month,
        day,
        hour,
        minute,
        second,
    } = Utc::from_unix(since.as_secs());

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{:03}Z",
        since.subsec_millis()
    )
}
