//! The lines of a `console` block and the words of its commands. A
//! command follows a `$ ` prompt, every other line is what a program printed.

#[cfg(test)]
mod tests;

const PROMPT: &str = "$ ";

/// One line of a `console` block, its line break included, so the parts
/// of a block add up to its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleLine<'text> {
    Command {
        prompt: &'text str,
        command: &'text str,
    },
    Output(&'text str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    /// The first word that is not an assignment.
    Program,
    Flag,
    /// The part of `key=value` up to and with its `=`.
    Key,
    Value,
    /// Subcommands, arguments, paths and the room between words.
    Plain,
}

/// A line is a command only when it starts with `$` and a space, so
/// `$HOME` at the start of a line is output.
pub fn console_lines(text: &str) -> impl Iterator<Item = ConsoleLine<'_>> {
    text.split_inclusive('\n').map(|line| {
        line.split_at_checked(PROMPT.len())
            .filter(|(prompt, _)| *prompt == PROMPT)
            .map_or(ConsoleLine::Output(line), |(prompt, command)| {
                ConsoleLine::Command { prompt, command }
            })
    })
}

/// The words and the room between them add up to `command`. Quotes,
/// pipes and comments are not read: the guide's commands have no word
/// they would change the colour of.
pub fn command_words(command: &str) -> Vec<(Word, &str)> {
    let mut parts = Vec::new();
    let mut program_seen = false;
    let mut rest = command;
    while let Some(first) = rest.chars().next() {
        let blank = first.is_whitespace();
        let end = rest
            .find(|next: char| next.is_whitespace() != blank)
            .unwrap_or(rest.len());

        let (piece, after) = rest.split_at(end);
        rest = after;
        if blank {
            parts.push((Word::Plain, piece));
        } else {
            word(piece, &mut program_seen, &mut parts);
        }
    }

    parts
}

fn word<'text>(piece: &'text str, program_seen: &mut bool, parts: &mut Vec<(Word, &'text str)>) {
    if piece.starts_with('-') {
        parts.push((Word::Flag, piece));
        return;
    }

    if let Some(at) = piece.find('=').filter(|at| *at > 0) {
        let (key, value) = piece.split_at(at + 1);
        parts.push((Word::Key, key));
        if !value.is_empty() {
            parts.push((Word::Value, value));
        }

        return;
    }

    let kind = if *program_seen {
        Word::Plain
    } else {
        Word::Program
    };

    *program_seen = true;
    parts.push((kind, piece));
}
