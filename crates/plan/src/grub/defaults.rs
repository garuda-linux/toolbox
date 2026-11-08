//! Reading and changing single variables of `/etc/default/grub`. The file
//! is a shell script that `grub-mkconfig` sources, so a value is whatever
//! a shell makes of one `NAME=value` line. Everything else stays byte for
//! byte: comments, blank lines, the order, and lines this parser cannot
//! read (commands, values over several lines).

#[cfg(test)]
mod tests;

use std::str::CharIndices;

use crate::lines::push_line;

const BLANKS: [char; 2] = [' ', '\t'];

/// What a shell acts on inside double quotes.
const ESCAPED_IN_QUOTES: [char; 4] = ['\\', '"', '$', '`'];

/// One `NAME=value` line, cut where a new value goes in.
struct Assignment<'line> {
    /// Up to and with the `=`.
    head: &'line str,
    token: Token,
    /// After the value: blanks, a comment, the line ending.
    tail: &'line str,
}

struct Token {
    length: usize,
    /// `None` when only a shell can tell (a `$` or a backtick).
    value: Option<String>,
    quoted: bool,
}

/// What the last line that sets `variable` gives it. `None` when no line
/// sets it, or when its value needs a shell to work out.
pub fn grub_value(config: &str, variable: &str) -> Option<String> {
    let found = config
        .split_inclusive('\n')
        .filter_map(|line| assignment(line, variable))
        .next_back()?;

    found.token.value
}

/// `config` with `variable` set to `value`. No other byte changes.
///
/// The last line that sets it gets the new value (a shell lets the last
/// one win), else the last commented-out line of it is switched on, else
/// a line is added at the end.
pub fn set_grub_value(config: &str, variable: &str, value: &str) -> String {
    let lines: Vec<&str> = config.split_inclusive('\n').collect();
    let active = lines
        .iter()
        .rposition(|line| assignment(line, variable).is_some());

    let dormant = || {
        lines
            .iter()
            .rposition(|line| commented(line, variable).is_some())
    };

    let Some(position) = active.or_else(dormant) else {
        return appended(config, variable, value);
    };

    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if index == position {
                rewritten(line, variable, value)
            } else {
                (*line).to_owned()
            }
        })
        .collect()
}

pub fn unset_grub_value(config: &str, variable: &str) -> String {
    let mut result = String::with_capacity(config.len());

    for line in config.split_inclusive('\n') {
        if assignment(line, variable).is_some() {
            let body = line.trim_start_matches(BLANKS);

            result.push_str(line.strip_suffix(body).unwrap_or_default());
            result.push('#');
            result.push_str(body);
        } else {
            result.push_str(line);
        }
    }

    result
}

/// `value` in double quotes, so that a shell reads exactly `value` again.
pub fn quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len().saturating_add(2));

    quoted.push('"');

    for character in value.chars() {
        if ESCAPED_IN_QUOTES.contains(&character) {
            quoted.push('\\');
        }

        quoted.push(character);
    }

    quoted.push('"');
    quoted
}

/// A commented-out line loses its `#`.
fn rewritten(line: &str, variable: &str, value: &str) -> String {
    let Some((indent, found)) = assignment(line, variable)
        .map(|found| ("", found))
        .or_else(|| commented(line, variable))
    else {
        return line.to_owned();
    };

    format!(
        "{indent}{}{}{}",
        found.head,
        written(value, found.token.quoted),
        found.tail
    )
}

fn appended(config: &str, variable: &str, value: &str) -> String {
    let mut result = config.to_owned();
    push_line(
        &mut result,
        &format!("{variable}={}", written(value, false)),
    );
    result
}

/// A plain word stays bare where the line had it bare (`GRUB_TIMEOUT=5`),
/// everything else gets double quotes.
fn written(value: &str, was_quoted: bool) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.-".contains(character));

    if plain && !was_quoted {
        value.to_owned()
    } else {
        quote(value)
    }
}

/// The line as an assignment of `variable`, with or without `export`.
/// `None` for other lines and for those a line-wise reader cannot trust:
/// a second command on the line, a value that goes on in the next one.
fn assignment<'line>(line: &'line str, variable: &str) -> Option<Assignment<'line>> {
    let body = line.trim_start_matches(BLANKS);
    let named = body
        .strip_prefix("export")
        .filter(|rest| rest.starts_with(BLANKS))
        .map_or(body, |rest| rest.trim_start_matches(BLANKS));

    let rest = named.strip_prefix(variable)?.strip_prefix('=')?;
    let head = line.strip_suffix(rest)?;
    let token = token(rest)?;
    let (_, tail) = rest.split_at_checked(token.length)?;
    let after = tail.trim_start();

    (after.is_empty() || after.starts_with('#')).then_some(Assignment { head, token, tail })
}

fn commented<'line>(line: &'line str, variable: &str) -> Option<(&'line str, Assignment<'line>)> {
    let body = line.trim_start_matches(BLANKS);
    let indent = line.strip_suffix(body)?;
    let inner = body
        .strip_prefix('#')?
        .trim_start_matches(|character: char| character == '#' || BLANKS.contains(&character));

    assignment(inner, variable).map(|found| (indent, found))
}

fn token(rest: &str) -> Option<Token> {
    let mut value = String::new();
    let mut known = true;
    let mut quoted = false;
    let mut length = rest.len();
    let mut characters = rest.char_indices();

    while let Some((at, character)) = characters.next() {
        match character {
            _ if character.is_whitespace() => {
                length = at;
                break;
            }
            '\'' => {
                quoted = true;
                single_quoted(&mut characters, &mut value)?;
            }
            '"' => {
                quoted = true;
                known &= double_quoted(&mut characters, &mut value)?;
            }
            '\\' => value.push(escaped(&mut characters)?),
            '$' | '`' => known = false,
            ';' | '&' | '|' | '(' | ')' | '<' | '>' => return None,
            _ => value.push(character),
        }
    }

    Some(Token {
        length,
        value: known.then_some(value),
        quoted,
    })
}

/// The character after a backslash. A line break there joins two lines,
/// which this reader does not follow.
fn escaped(characters: &mut CharIndices<'_>) -> Option<char> {
    characters
        .next()
        .map(|(_, character)| character)
        .filter(|character| !matches!(character, '\n' | '\r'))
}

fn single_quoted(characters: &mut CharIndices<'_>, value: &mut String) -> Option<()> {
    loop {
        match characters.next()?.1 {
            '\'' => return Some(()),
            character => value.push(character),
        }
    }
}

/// Returns whether the value is known without a shell.
fn double_quoted(characters: &mut CharIndices<'_>, value: &mut String) -> Option<bool> {
    let mut known = true;

    loop {
        match characters.next()?.1 {
            '"' => return Some(known),
            '\\' => {
                let character = escaped(characters)?;

                if !ESCAPED_IN_QUOTES.contains(&character) {
                    value.push('\\');
                }

                value.push(character);
            }
            '$' | '`' => known = false,
            character => value.push(character),
        }
    }
}
