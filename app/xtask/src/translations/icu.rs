//! Text, variables and plurals as ICU `MessageFormat`, the part Transifex
//! reads in Structured JSON. Apostrophes follow ICU's optional doubling: a
//! quote starts quoting only before a brace, a quote, or `#` in a plural.

#[cfg(test)]
mod tests;

use crate::translations::model::{Part, Plural};

struct Reader {
    chars: Vec<char>,
    at: usize,
}

impl Reader {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    /// Reads until the end, or until the `}` closing a plural branch,
    /// which it leaves for the caller. `plural` is the plural's variable.
    fn message(&mut self, plural: Option<&str>) -> Result<Vec<Part>, String> {
        let mut parts: Vec<Part> = Vec::new();
        let mut text = String::new();

        while let Some(character) = self.peek() {
            match character {
                '\'' => self.quote(&mut text, plural.is_some()),
                '{' => {
                    push_text(&mut parts, &mut text);
                    parts.push(self.argument()?);
                }
                '}' if plural.is_some() => break,
                '}' => return Err("has an unbalanced closing brace".to_owned()),
                '#' if plural.is_some() => {
                    push_text(&mut parts, &mut text);
                    parts.push(Part::Var(plural.unwrap_or_default().to_owned()));
                    self.at += 1;
                }
                _ => {
                    text.push(character);
                    self.at += 1;
                }
            }
        }

        push_text(&mut parts, &mut text);
        Ok(parts)
    }

    fn quote(&mut self, text: &mut String, plural: bool) {
        self.at += 1;

        match self.peek() {
            Some('\'') => {
                text.push('\'');
                self.at += 1;
            }
            Some(after) if matches!(after, '{' | '}' | '|') || (plural && after == '#') => {
                while let Some(quoted) = self.peek() {
                    self.at += 1;

                    if quoted != '\'' {
                        text.push(quoted);
                    } else if self.peek() == Some('\'') {
                        text.push('\'');
                        self.at += 1;
                    } else {
                        break;
                    }
                }
            }
            Some(_) | None => text.push('\''),
        }
    }

    fn argument(&mut self) -> Result<Part, String> {
        self.at += 1;
        let name = self.word();
        self.skip_space();

        match self.peek() {
            Some('}') => {
                self.at += 1;
                Ok(Part::Var(name))
            }
            Some(',') => {
                self.at += 1;
                self.skip_space();
                let kind = self.word();
                self.skip_space();

                match kind.as_str() {
                    "plural" if self.peek() == Some(',') => {
                        self.at += 1;
                        self.plural(name)
                    }
                    "select" => Err("has an ICU select, which Fluent here cannot hold".to_owned()),
                    other => Err(format!("has an ICU `{other}` argument (plain, number)")),
                }
            }
            Some(_) | None => Err("has a brace that is never closed".to_owned()),
        }
    }

    fn plural(&mut self, var: String) -> Result<Part, String> {
        let mut variants = Vec::new();

        loop {
            self.skip_space();

            match self.peek() {
                Some('}') => {
                    self.at += 1;
                    break;
                }
                Some('=') => return Err("has an exact plural key such as =1".to_owned()),
                None => return Err("has a brace that is never closed".to_owned()),
                Some(_) => {}
            }

            let key = self.word();
            if key.starts_with("offset") {
                return Err("has a plural offset".to_owned());
            }

            self.skip_space();
            if self.peek() != Some('{') {
                return Err(format!("has a plural key `{key}` without a branch"));
            }

            self.at += 1;
            let branch = self.message(Some(&var))?;
            if self.peek() != Some('}') {
                return Err("has a brace that is never closed".to_owned());
            }

            self.at += 1;
            variants.push((key, branch));
        }

        if !variants.iter().any(|(key, _)| key == "other") {
            return Err("has a plural without `other`".to_owned());
        }

        Ok(Part::Plural(Plural { var, variants }))
    }

    fn word(&mut self) -> String {
        self.skip_space();
        let mut word = String::new();

        while let Some(character) = self.peek() {
            if character.is_alphanumeric() || matches!(character, '_' | '-' | ':') {
                word.push(character);
                self.at += 1;
            } else {
                break;
            }
        }

        word
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.at += 1;
        }
    }
}

pub fn icu(parts: &[Part]) -> String {
    let mut out = String::new();
    write(&mut out, parts, false, false);
    out
}

/// `plural` is whether `#` is special here, `then_syntax` whether ICU
/// syntax follows these parts (a closing brace of a plural branch).
fn write(out: &mut String, parts: &[Part], plural: bool, then_syntax: bool) {
    for (at, part) in parts.iter().enumerate() {
        let syntax_next = parts.get(at + 1).map_or(then_syntax, |next| {
            matches!(next, Part::Var(_) | Part::Plural(_))
        });

        match part {
            Part::Text(text) => write_text(out, text, plural, syntax_next),
            Part::Var(name) => {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
            Part::Plural(Plural { var, variants }) => {
                out.push('{');
                out.push_str(var);
                out.push_str(", plural,");

                for (key, branch) in variants {
                    out.push(' ');
                    out.push_str(key);
                    out.push_str(" {");
                    write(out, branch, true, true);
                    out.push('}');
                }

                out.push('}');
            }
        }
    }
}

fn write_text(out: &mut String, text: &str, plural: bool, syntax_next: bool) {
    let chars: Vec<char> = text.chars().collect();

    for (at, &character) in chars.iter().enumerate() {
        let next = chars.get(at + 1).copied();
        let special = |after: char| matches!(after, '{' | '}' | '\'') || (plural && after == '#');

        match character {
            '{' | '}' => {
                out.push('\'');
                out.push(character);
                out.push('\'');
            }
            '#' if plural => out.push_str("'#'"),
            '\'' if next.map_or(syntax_next, special) => out.push_str("''"),
            _ => out.push(character),
        }
    }
}

/// Reads an ICU message back. Refuses what Fluent here cannot hold:
/// select, offsets, exact plural keys and other argument types.
pub fn parse(written: &str) -> Result<Vec<Part>, String> {
    let chars: Vec<char> = written.chars().collect();
    let mut reader = Reader { chars, at: 0 };
    let parts = reader.message(None)?;

    if reader.at < reader.chars.len() {
        return Err("has an unbalanced closing brace".to_owned());
    }

    Ok(parts)
}

fn push_text(parts: &mut Vec<Part>, text: &mut String) {
    if !text.is_empty() {
        parts.push(Part::Text(std::mem::take(text)));
    }
}
