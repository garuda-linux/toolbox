#[cfg(test)]
mod tests;

/// The text of a system file. One byte that is no UTF-8, such as a
/// Latin-1 full name in `/etc/passwd`, must not hide every other line, and
/// the helper and the app have to read the same entries.
pub fn lossy(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

/// The lines of a system list that hold an entry, without blanks around.
pub fn entries(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

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

/// `config` with every line `replace` takes swapped for `wanted`, each
/// keeping its own break. The flag tells whether any line was taken.
pub fn replace_lines(
    config: &str,
    wanted: &str,
    mut replace: impl FnMut(&str) -> bool,
) -> (String, bool) {
    let mut placed = false;
    let mut result = String::with_capacity(config.len() + wanted.len() + 1);

    for line in config.split_inclusive('\n') {
        if replace(line) {
            placed = true;
            result.push_str(wanted);
            result.push_str(ending(line));
        } else {
            result.push_str(line);
        }
    }

    (result, placed)
}
