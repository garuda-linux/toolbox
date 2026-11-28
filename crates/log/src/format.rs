use jiff::Timestamp;
use jiff::tz::TimeZone;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Plain,
    Color,
}

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
/// Hidden characters are dropped: a message that quotes program output
/// could otherwise drive the terminal of whoever reads the log. A carriage
/// return goes too, or the text after it could hide the start of its line.
pub fn format_record(
    time: Timestamp,
    level: log::Level,
    target: &str,
    message: &str,
    style: Style,
) -> String {
    let printable = toolbox_text::OutsideText::keep_lines(message);

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
fn timestamp(time: Timestamp) -> String {
    let since = time.max(Timestamp::UNIX_EPOCH);
    let utc = since.to_zoned_datetime(TimeZone::UTC);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        utc.year(),
        utc.month(),
        utc.day(),
        utc.hour(),
        utc.minute(),
        utc.second(),
        since.subsec_nanosecond() / 1_000_000
    )
}
