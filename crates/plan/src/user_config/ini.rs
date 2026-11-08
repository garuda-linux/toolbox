//! One key of a KDE style INI file (`[Group]` lines, `key=value` entries,
//! `#` comments). Every other byte of the file stays as it was.

#[cfg(test)]
mod tests;

use crate::lines::push_line;

/// The value of `key` in `group`. A group may appear more than once and a
/// later entry wins, as `KConfig` reads it.
pub fn value<'text>(text: &'text str, group: &str, key: &str) -> Option<&'text str> {
    let mut in_group = false;
    let mut found = None;

    for line in text.lines() {
        match header(line) {
            Some(name) => in_group = name == group,
            None if in_group => found = entry_value(line, key).or(found),
            None => {}
        }
    }

    found
}

/// `text` with `key=value` in `group`. Existing entries of the key in the
/// group are replaced in place. Without one the entry goes after the
/// group's last line, and without the group a new group is appended.
pub fn set(text: &str, group: &str, key: &str, value: &str) -> String {
    let entry = format!("{key}={value}");
    let mut out = String::with_capacity(text.len() + entry.len() + group.len() + 4);
    let mut in_group = false;
    let mut replaced = false;
    let mut group_end = None;

    for line in text.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        let ending = line.strip_prefix(content).unwrap_or_default();

        match header(content) {
            Some(name) => {
                in_group = name == group;
                out.push_str(line);
            }
            None if in_group && entry_value(content, key).is_some() => {
                out.push_str(&entry);
                out.push_str(ending);
                replaced = true;
            }
            None => out.push_str(line),
        }

        if in_group && !content.trim().is_empty() {
            group_end = Some(out.len());
        }
    }

    if !replaced {
        add(&mut out, group_end, group, &entry);
    }

    out
}

/// Puts `entry` at `group_end`, or appends the group with it after a blank
/// line.
fn add(out: &mut String, group_end: Option<usize>, group: &str, entry: &str) {
    if let Some(end) = group_end {
        let after_newline = out.get(..end).is_some_and(|before| before.ends_with('\n'));
        let separator = if after_newline { "" } else { "\n" };

        out.insert_str(end, &format!("{separator}{entry}\n"));
        return;
    }

    if !out.is_empty() && !out.ends_with("\n\n") {
        push_line(out, "");
    }

    push_line(out, &format!("[{group}]"));
    push_line(out, entry);
}

fn header(line: &str) -> Option<&str> {
    line.trim().strip_prefix('[')?.strip_suffix(']')
}

/// The value of a `key=value` line for `key`, blanks around both trimmed.
fn entry_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    let line = line.trim_start();

    if line.starts_with('#') {
        return None;
    }

    let (name, value) = line.split_once('=')?;

    (name.trim_end() == key).then(|| value.trim())
}
