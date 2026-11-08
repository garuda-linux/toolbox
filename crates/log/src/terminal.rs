use std::io::{IsTerminal as _, Write as _};
use std::time::SystemTime;

use crate::format::{Style, format_record};

pub fn echo(time: SystemTime, level: log::Level, target: &str, message: &str) {
    // Colours only on a real terminal, and never with NO_COLOR (no-color.org).
    let terminal = std::io::stderr().is_terminal()
        && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty());

    let style = if terminal { Style::Color } else { Style::Plain };
    let line = format_record(time, level, target, message, style);

    // Nothing sensible to do when the terminal is gone.
    std::io::stderr().write_all(line.as_bytes()).ok();
}
