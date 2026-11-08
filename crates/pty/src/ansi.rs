//! ANSI escape sequences in program output, removed for logs and plain
//! reports.

#[cfg(test)]
mod tests;

use std::iter::Peekable;
use std::str::Chars;

pub const ESCAPE: char = '\u{1b}';

/// Whether `character` is text a view may show or a program may receive
/// as typed: anything but a control character, except the tab and the
/// line breaks.
pub fn is_plain_text_char(character: char) -> bool {
    !character.is_control() || matches!(character, '\t' | '\n' | '\r')
}

/// Remove ANSI escape sequences (colours, cursor movement, titles) from
/// `text`. An unfinished sequence at the end is dropped too.
///
/// Other control characters go as well, except the tab and the line
/// breaks: the result is copied and uploaded as plain text, and a stray
/// one would still drive the terminal that shows it.
#[must_use = "the stripped text is returned, the input is unchanged"]
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == ESCAPE {
            read_escape(&mut chars, None);
            continue;
        }

        if is_plain_text_char(ch) {
            out.push(ch);
        }
    }

    out
}

/// Read the rest of an escape sequence after ESC, into `sequence` when
/// the caller wants it.
pub fn read_escape(chars: &mut Peekable<Chars<'_>>, mut sequence: Option<&mut String>) {
    let mut keep = |character| {
        if let Some(sequence) = sequence.as_deref_mut() {
            sequence.push(character);
        }
    };

    let Some(kind) = chars.next() else {
        return;
    };

    keep(kind);

    match kind {
        // CSI: parameters, then a final byte.
        '[' => {
            for next in chars.by_ref() {
                keep(next);
                if ('@'..='~').contains(&next) {
                    return;
                }
            }
        }
        // OSC (window titles, hyperlinks) ends with BEL or ESC \.
        ']' => {
            while let Some(next) = chars.next() {
                keep(next);
                if next == '\u{7}' {
                    return;
                }

                if next == ESCAPE && chars.peek() == Some(&'\\') {
                    chars.next();
                    return;
                }
            }
        }
        // tput sgr0 selects a character set this way:
        // intermediate bytes, then one final byte.
        ' '..='/' => {
            for next in chars.by_ref() {
                keep(next);
                if !(' '..='/').contains(&next) {
                    return;
                }
            }
        }
        _ => {}
    }
}
