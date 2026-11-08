#[cfg(test)]
mod tests;

pub fn ending(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

/// Appends `line` and its break, after ending a last line that has none.
pub fn push_line(text: &mut String, line: &str) {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }

    text.push_str(line);
    text.push('\n');
}
