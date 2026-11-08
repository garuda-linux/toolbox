//! Words of comment prose and of code. Verbs fold into concepts ("Returns",
//! "fetches" and "loads" all become `get`), so a comment that restates the
//! code with a synonym still matches it.

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::pattern::re;
use crate::source::Leaf;

/// A comment that gives a reason is never just a restatement.
pub static WHY_WORDS: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"(?i)\b(because|since|so that|otherwise|avoid|prevent|workaround|hack|bug|must|needs?|required?|instead|in case|due to|why|unless|only when|quirk)\b",
    )
});

const STOPWORDS: &[&str] = &[
    "a",
    "an",
    "the",
    "to",
    "of",
    "for",
    "and",
    "or",
    "is",
    "are",
    "be",
    "this",
    "that",
    "it",
    "its",
    "with",
    "in",
    "on",
    "by",
    "as",
    "at",
    "from",
    "into",
    "all",
    "over",
    "given",
    "new",
    "specified",
    "provided",
    "current",
    "then",
    "program",
];

const CONCEPTS: &[(&str, &[&str])] = &[
    (
        "get",
        &[
            "get",
            "gets",
            "fetch",
            "fetches",
            "retrieve",
            "retrieves",
            "load",
            "loads",
            "read",
            "reads",
            "obtain",
            "obtains",
            "return",
            "returns",
        ],
    ),
    (
        "set",
        &[
            "set",
            "sets",
            "assign",
            "assigns",
            "store",
            "stores",
            "update",
            "updates",
            "reset",
            "resets",
            "init",
            "initialize",
            "initializes",
            "initialise",
        ],
    ),
    (
        "create",
        &[
            "create",
            "creates",
            "build",
            "builds",
            "make",
            "makes",
            "generate",
            "generates",
            "construct",
            "constructs",
        ],
    ),
    (
        "check",
        &[
            "check",
            "checks",
            "verify",
            "verifies",
            "validate",
            "validates",
            "whether",
            "if",
            "is",
            "has",
        ],
    ),
    (
        "loop",
        &[
            "loop", "loops", "iterate", "iterates", "each", "every", "for", "foreach", "while",
        ],
    ),
    (
        "remove",
        &[
            "remove",
            "removes",
            "delete",
            "deletes",
            "clear",
            "clears",
            "erase",
            "erases",
            "wipe",
            "wipes",
            "truncate",
            "truncates",
            "close",
            "closes",
            "drop",
            "drops",
            "rename",
            "renames",
            "renaming",
            "rotate",
            "rotates",
        ],
    ),
    (
        "emit",
        &[
            "emit",
            "emits",
            "apply",
            "applies",
            "feed",
            "feeds",
            "write",
            "writes",
            "show",
            "shows",
            "display",
            "displays",
            "render",
            "renders",
            "stay",
            "stays",
            "keep",
            "keeps",
            "preserve",
            "preserves",
        ],
    ),
    (
        "split",
        &[
            "split",
            "splits",
            "divide",
            "divides",
            "separate",
            "separates",
        ],
    ),
    (
        "file",
        &[
            "file",
            "files",
            "path",
            "paths",
            "directory",
            "directories",
            "folder",
            "folders",
        ],
    ),
    ("output", &["output", "outputs", "text", "texts"]),
    (
        "call",
        &[
            "call", "calls", "invoke", "invokes", "run", "runs", "execute", "executes",
        ],
    ),
];

const TOKEN_CONCEPTS: &[(&str, &[&str])] = &[
    ("=", &["set", "get"]),
    ("+=", &["set"]),
    ("-=", &["set"]),
    ("if", &["check"]),
    ("match", &["check"]),
    ("?", &["check"]),
    ("==", &["check"]),
    ("!=", &["check"]),
    ("for", &["loop"]),
    ("while", &["loop"]),
    ("loop", &["loop"]),
    ("return", &["get"]),
    ("await", &["call"]),
    ("new", &["create"]),
    ("(", &["call"]),
];

const ABBREVIATIONS: &[(&str, &str)] = &[
    ("num", "number"),
    ("nr", "number"),
    ("no", "number"),
    ("qty", "quantity"),
    ("amt", "amount"),
    ("desc", "description"),
    ("cfg", "configuration"),
    ("config", "configuration"),
    ("db", "database"),
    ("doc", "document"),
    ("msg", "message"),
    ("idx", "index"),
    ("btn", "button"),
    ("val", "value"),
    ("str", "string"),
    ("arr", "array"),
    ("obj", "object"),
    ("err", "error"),
    ("req", "request"),
    ("res", "response"),
    ("param", "parameter"),
    ("id", "identifier"),
];

pub fn identifier_words(name: &str) -> Vec<String> {
    let mut spaced = String::with_capacity(name.len() + 8);
    let chars: Vec<char> = name.chars().collect();

    for (index, current) in chars.iter().enumerate() {
        let previous = index.checked_sub(1).and_then(|at| chars.get(at));
        let next = chars.get(index + 1);
        let lower_to_upper = previous
            .is_some_and(|before| before.is_lowercase() || before.is_ascii_digit())
            && current.is_uppercase();

        let acronym_end = previous.is_some_and(|before| before.is_uppercase())
            && current.is_uppercase()
            && next.is_some_and(|after| after.is_lowercase());

        if lower_to_upper || acronym_end {
            spaced.push(' ');
        }

        spaced.push(*current);
    }

    spaced
        .split(|separator: char| !separator.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(normalize)
        .collect()
}

pub fn prose_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|separator: char| !separator.is_ascii_lowercase() && !separator.is_ascii_digit())
        .filter(|word| !word.is_empty() && !STOPWORDS.contains(word))
        .map(normalize)
        .collect()
}

pub fn code_words(leaves: &[Leaf<'_>]) -> HashSet<String> {
    let mut words = HashSet::new();

    for leaf in leaves {
        if let Some((_, concepts)) = TOKEN_CONCEPTS.iter().find(|(token, _)| *token == leaf.text) {
            words.extend(concepts.iter().map(|&concept| concept.to_owned()));
        }

        let keyword = !leaf.named && leaf.text.chars().all(char::is_alphabetic);
        if leaf.kind.contains("identifier") || keyword || is_literal_word(leaf.kind) {
            words.extend(identifier_words(leaf.text));
        }
    }

    words
}

pub fn mostly_covered(words: &[String], known: &HashSet<String>) -> bool {
    let covered = words.iter().filter(|word| known.contains(*word)).count();
    !words.is_empty() && covered * 4 >= words.len() * 3
}

fn is_literal_word(kind: &str) -> bool {
    matches!(kind, "true" | "false" | "boolean_literal" | "self")
}

fn normalize(word: &str) -> String {
    let lower = word.to_lowercase();
    let full = ABBREVIATIONS
        .iter()
        .find(|(short, _)| *short == lower)
        .map_or(lower.as_str(), |(_, long)| long);

    if let Some((concept, _)) = CONCEPTS.iter().find(|(_, words)| words.contains(&full)) {
        return (*concept).to_owned();
    }

    let plural = full.len() > 3 && full.ends_with('s') && !full.ends_with("ss");
    let stem = if plural {
        full.strip_suffix('s').unwrap_or(full)
    } else {
        full
    };

    stem.to_owned()
}
