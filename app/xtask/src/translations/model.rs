//! A Fluent message as the part of Fluent that Transifex can hold: text,
//! variables and plurals, or a choice on other words that fills the whole
//! message. Everything else is refused with its reason and stays out of
//! Transifex.

#[cfg(test)]
mod tests;

use fluent_syntax::ast::{
    Entry, Expression, InlineExpression, Pattern, PatternElement, Variant, VariantKey,
};

/// The plural categories of CLDR, the keys an ICU plural knows.
const PLURAL_KEYS: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    Text(String),
    Var(String),
    Plural(Plural),
}

/// `variants` in file order. `other` is the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plural {
    pub var: String,
    pub variants: Vec<(String, Vec<Part>)>,
}

/// A choice on other words than plural categories, the whole message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub var: String,
    pub default: String,
    pub variants: Vec<(String, Vec<Part>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Plain(Vec<Part>),
    Choice(Choice),
}

/// One message of a file: its id, the comment right above it and its
/// value, or why the value cannot go to Transifex.
#[derive(Debug)]
pub struct Message {
    pub id: String,
    pub note: Option<String>,
    pub value: Result<Value, String>,
}

/// Every message of `source` in file order. Fails with the parser's
/// complaint when the file is no valid Fluent.
pub fn messages(source: &str) -> Result<Vec<Message>, String> {
    let resource = fluent_syntax::parser::parse(source)
        .map_err(|(_, errors)| format!("not valid Fluent: {errors:?}"))?;

    Ok(resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            Entry::Message(message) => Some(message),
            Entry::Term(_)
            | Entry::Comment(_)
            | Entry::GroupComment(_)
            | Entry::ResourceComment(_)
            | Entry::Junk { content: _ } => None,
        })
        .map(|message| {
            let value = if message.attributes.is_empty() {
                message
                    .value
                    .as_ref()
                    .map_or_else(|| Err("has no value".to_owned()), value)
            } else {
                Err("has an attribute".to_owned())
            };

            Message {
                id: message.id.name.to_owned(),
                note: message
                    .comment
                    .as_ref()
                    .map(|comment| comment.content.join("\n")),
                value,
            }
        })
        .collect())
}

fn value(pattern: &Pattern<&str>) -> Result<Value, String> {
    if let [PatternElement::Placeable { expression }] = pattern.elements.as_slice()
        && let Expression::Select { selector, variants } = expression
        && !is_plural(variants)
    {
        return Ok(Value::Choice(Choice {
            var: variable(selector)?,
            default: default_key(variants)?,
            variants: variants
                .iter()
                .map(|variant| Ok((key(variant)?, parts(&variant.value, true)?)))
                .collect::<Result<_, String>>()?,
        }));
    }

    parts(pattern, true).map(Value::Plain)
}

/// `plural` is whether a plural may still come: never inside another one.
fn parts(pattern: &Pattern<&str>, plural: bool) -> Result<Vec<Part>, String> {
    let mut found: Vec<Part> = Vec::new();

    for element in &pattern.elements {
        let part = match element {
            PatternElement::TextElement { value } => Part::Text((*value).to_owned()),
            PatternElement::Placeable { expression } => placeable(expression, plural)?,
        };

        match (found.last_mut(), part) {
            (Some(Part::Text(before)), Part::Text(more)) => before.push_str(&more),
            (_, part) => found.push(part),
        }
    }

    Ok(found)
}

fn placeable(expression: &Expression<&str>, plural: bool) -> Result<Part, String> {
    match expression {
        Expression::Inline(inline) => inline_part(inline),
        Expression::Select { selector, variants } if is_plural(variants) => {
            if !plural {
                return Err("has a nested choice".to_owned());
            }

            Ok(Part::Plural(Plural {
                var: variable(selector)?,
                variants: variants
                    .iter()
                    .map(|variant| Ok((key(variant)?, parts(&variant.value, false)?)))
                    .collect::<Result<_, String>>()?,
            }))
        }
        Expression::Select {
            selector: _,
            variants,
        } if variants
            .iter()
            .any(|variant| matches!(variant.key, VariantKey::NumberLiteral { value: _ })) =>
        {
            Err("has a number as a choice key".to_owned())
        }
        Expression::Select {
            selector: _,
            variants: _,
        } => Err(if plural {
            "has a choice inside its text".to_owned()
        } else {
            "has a nested choice".to_owned()
        }),
    }
}

fn inline_part(inline: &InlineExpression<&str>) -> Result<Part, String> {
    match inline {
        InlineExpression::StringLiteral { value } => Ok(Part::Text(unescape(value)?)),
        InlineExpression::VariableReference { id } => Ok(Part::Var(id.name.to_owned())),
        InlineExpression::FunctionReference {
            id: _,
            arguments: _,
        } => Err("calls a function".to_owned()),
        InlineExpression::MessageReference {
            id: _,
            attribute: _,
        } => Err("uses another message".to_owned()),
        InlineExpression::TermReference {
            id: _,
            attribute: _,
            arguments: _,
        } => Err("uses a term".to_owned()),
        InlineExpression::NumberLiteral { value: _ } => Err("has a number literal".to_owned()),
        InlineExpression::Placeable { expression: _ } => Err("has a nested placeable".to_owned()),
    }
}

fn is_plural(variants: &[Variant<&str>]) -> bool {
    variants.iter().all(|variant| {
        matches!(variant.key, VariantKey::Identifier { name } if PLURAL_KEYS.contains(&name))
    }) && default_key(variants).is_ok_and(|name| name == "other")
}

fn variable(selector: &InlineExpression<&str>) -> Result<String, String> {
    match selector {
        InlineExpression::VariableReference { id } => Ok(id.name.to_owned()),
        InlineExpression::StringLiteral { value: _ }
        | InlineExpression::NumberLiteral { value: _ }
        | InlineExpression::FunctionReference {
            id: _,
            arguments: _,
        }
        | InlineExpression::MessageReference {
            id: _,
            attribute: _,
        }
        | InlineExpression::TermReference {
            id: _,
            attribute: _,
            arguments: _,
        }
        | InlineExpression::Placeable { expression: _ } => {
            Err("chooses on something else than a variable".to_owned())
        }
    }
}

fn key(variant: &Variant<&str>) -> Result<String, String> {
    match variant.key {
        VariantKey::Identifier { name } => Ok(name.to_owned()),
        VariantKey::NumberLiteral { value: _ } => Err("has a number as a choice key".to_owned()),
    }
}

fn default_key(variants: &[Variant<&str>]) -> Result<String, String> {
    variants
        .iter()
        .find(|variant| variant.default)
        .map_or_else(|| Err("has a choice without a default".to_owned()), key)
}

/// A Fluent string literal's escapes: `\\`, `\"`, `\uXXXX`, `\UXXXXXX`.
fn unescape(raw: &str) -> Result<String, String> {
    let mut text = String::with_capacity(raw.len());
    let mut chars = raw.chars();

    while let Some(character) = chars.next() {
        if character != '\\' {
            text.push(character);
            continue;
        }

        let width = match chars.next() {
            Some(escaped @ ('\\' | '"')) => {
                text.push(escaped);
                continue;
            }
            Some('u') => 4,
            Some('U') => 6,
            Some(_) | None => return Err("has an unknown escape".to_owned()),
        };

        let digits: String = chars.by_ref().take(width).collect();
        let decoded = u32::from_str_radix(&digits, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| "has an unknown escape".to_owned())?;

        text.push(decoded);
    }

    Ok(text)
}
