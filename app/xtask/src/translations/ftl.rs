//! Writes a value as Fluent and puts it into a file, line by line, so
//! every message an import does not change keeps its bytes.

#[cfg(test)]
mod tests;

use crate::catalog_ftl::UNTRANSLATED;
use crate::translations::model::{Choice, Part, Plural, Value};

const INDENT: &str = "    ";

#[derive(Debug)]
pub enum Edit {
    /// Puts `lines` in place of the message `id` and of its untranslated
    /// mark.
    Replace { id: String, lines: String },
    /// Adds `lines` after the message `after`, or at the end.
    Insert {
        after: Option<String>,
        lines: String,
    },
}

/// A message's lines in a file: its first line, the one after its last,
/// and the line of its untranslated mark.
struct Span {
    id: String,
    start: usize,
    end: usize,
    mark: Option<usize>,
}

/// `id` with `value`, without a final line break.
pub fn render(id: &str, value: &Value) -> String {
    match value {
        Value::Plain(parts) if parts.iter().all(|part| !is_block(part)) => {
            format!("{id} = {}", inline(parts, true, true))
        }
        Value::Plain(parts) => format!("{id} =\n{INDENT}{}", inline(parts, true, true)),
        Value::Choice(Choice {
            var,
            default,
            variants,
        }) => format!(
            "{id} =\n{INDENT}{}",
            select(var, variants, |key| key == default)
        ),
    }
}

fn is_block(part: &Part) -> bool {
    match part {
        Part::Text(text) => text.contains('\n'),
        Part::Var(_) => false,
        Part::Plural(_) => true,
    }
}

/// `first` and `last`: whether these parts start and end the pattern,
/// where Fluent drops spaces.
fn inline(parts: &[Part], first: bool, last: bool) -> String {
    let mut out = String::new();

    for (at, part) in parts.iter().enumerate() {
        match part {
            Part::Text(text) => {
                let starts = first && at == 0;
                let ends = last && at + 1 == parts.len();
                out.push_str(&text_value(text, starts, ends));
            }
            Part::Var(name) => {
                out.push_str("{ $");
                out.push_str(name);
                out.push_str(" }");
            }
            Part::Plural(Plural { var, variants }) => {
                out.push_str(&select(var, variants, |key| key == "other"));
            }
        }
    }

    out
}

fn select(var: &str, variants: &[(String, Vec<Part>)], default: impl Fn(&str) -> bool) -> String {
    let mut out = format!("{{ ${var} ->");
    let continued = format!("\n{INDENT}{INDENT}{INDENT}");

    for (key, branch) in variants {
        let mark = if default(key) { "       *" } else { "        " };
        let value = inline(branch, true, true).replace('\n', &continued);

        out.push('\n');
        out.push_str(mark);
        out.push('[');
        out.push_str(key);
        out.push_str("] ");
        out.push_str(&value);
    }

    out.push('\n');
    out.push_str(INDENT);
    out.push('}');
    out
}

/// A whole message value of plain text, escaped as [`text_value`] does.
pub fn plain_value(text: &str) -> String {
    text_value(text, true, true)
}

/// Text as a Fluent value: braces, a line's leading `[`, `*` or `.`, and
/// spaces Fluent would drop become string literals.
fn text_value(text: &str, starts: bool, ends: bool) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let count = lines.len();

    lines
        .iter()
        .enumerate()
        .map(|(number, line)| {
            let line_starts = number > 0 || starts;
            let line_ends = number + 1 == count && ends;
            escape_line(line, line_starts, line_ends)
        })
        .collect::<Vec<_>>()
        .join(&format!("\n{INDENT}"))
}

fn escape_line(line: &str, starts: bool, ends: bool) -> String {
    let body = line.trim_start_matches(' ');
    let lead = line.strip_suffix(body).unwrap_or_default();
    let core = if ends {
        body.trim_end_matches(' ')
    } else {
        body
    };

    let tail = body.strip_prefix(core).unwrap_or_default();

    let mut out = String::new();
    if !lead.is_empty() {
        out.push_str(&if starts {
            literal(lead)
        } else {
            lead.to_owned()
        });
    }

    for (at, character) in core.chars().enumerate() {
        let leading = at == 0 && starts && lead.is_empty() && matches!(character, '[' | '*' | '.');
        if leading || matches!(character, '{' | '}') {
            out.push_str(&literal(&character.to_string()));
        } else {
            out.push(character);
        }
    }

    if !tail.is_empty() {
        out.push_str(&literal(tail));
    }

    out
}

fn literal(text: &str) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"{escaped}\"}}")
}

fn message_id(line: &str) -> Option<&str> {
    let (head, _) = line.split_once('=')?;
    let id = head.trim_end();
    let valid = id.starts_with(|first: char| first.is_ascii_alphabetic())
        && id
            .chars()
            .all(|part| part.is_ascii_alphanumeric() || part == '-' || part == '_');

    valid.then_some(id)
}

fn spans(lines: &[&str]) -> Vec<Span> {
    let mut found = Vec::new();
    let mut at = 0;

    while let Some(line) = lines.get(at) {
        let Some(id) = message_id(line) else {
            at += 1;
            continue;
        };

        let mut end = at + 1;
        while lines
            .get(end)
            .is_some_and(|next| next.starts_with(' ') || next.starts_with('}'))
        {
            end += 1;
        }

        let mark = at
            .checked_sub(1)
            .filter(|before| lines.get(*before) == Some(&UNTRANSLATED));

        found.push(Span {
            id: id.to_owned(),
            start: at,
            end,
            mark,
        });
        at = end;
    }

    found
}

pub fn marked(file: &str) -> Vec<String> {
    let lines: Vec<&str> = file.lines().collect();

    spans(&lines)
        .into_iter()
        .filter(|span| span.mark.is_some())
        .map(|span| span.id)
        .collect()
}

/// The messages not marked untranslated, each with its value as it
/// stands in the file.
pub fn unmarked_values(file: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = file.lines().collect();

    spans(&lines)
        .into_iter()
        .filter(|span| span.mark.is_none())
        .filter_map(|span| {
            let (_, first) = lines.get(span.start)?.split_once('=')?;
            let rest = lines.get(span.start + 1..span.end)?;
            let value = std::iter::once(first.strip_prefix(' ').unwrap_or(first))
                .chain(rest.iter().copied())
                .collect::<Vec<_>>()
                .join("\n");

            Some((span.id, value))
        })
        .collect()
}

/// `file` with `edits` applied. A message not in the file is skipped.
pub fn apply(file: &str, edits: &[Edit]) -> String {
    let lines: Vec<&str> = file.lines().collect();
    let found = spans(&lines);
    let mut replaced: Vec<(usize, usize, &str)> = Vec::new();
    let mut inserted: Vec<(usize, &str)> = Vec::new();

    for edit in edits {
        match edit {
            Edit::Replace { id, lines: new } => {
                if let Some(span) = found.iter().find(|span| span.id == *id) {
                    replaced.push((span.mark.unwrap_or(span.start), span.end, new));
                }
            }
            Edit::Insert { after, lines: new } => {
                let place = after
                    .as_ref()
                    .and_then(|id| found.iter().find(|span| span.id == *id))
                    .map_or(lines.len(), |span| span.end);

                inserted.push((place, new));
            }
        }
    }

    let mut out = String::new();
    let mut at = 0;

    loop {
        for (_, new) in inserted.iter().filter(|(place, _)| *place == at) {
            out.push_str(new);
            out.push('\n');
        }

        let Some(line) = lines.get(at) else {
            break;
        };

        if let Some((_, end, new)) = replaced.iter().find(|(start, _, _)| *start == at) {
            out.push_str(new);
            out.push('\n');
            at = *end;
            continue;
        }

        out.push_str(line);
        out.push('\n');
        at += 1;
    }

    out
}
