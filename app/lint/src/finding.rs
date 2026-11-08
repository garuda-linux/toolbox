//! What a rule reports, in the problem / why / how-to-fix shape, so an agent
//! reading the output gets the reason and a next step, not just the violation.

use std::fmt;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use tree_sitter::Point;

use crate::pattern::re;
use crate::rules::NAMES;
use crate::source::Source;

/// `toolbox-lint: allow(rule-a, rule-b) reason`, on the reported line or
/// the line above it.
static ALLOW: LazyLock<Regex> =
    LazyLock::new(|| re(r"^\s*toolbox-lint:\s*allow\(([a-z0-9\-,\s]+)\)\s*(.*)$"));

pub const WITHOUT_REASON: &str = "allow-without-reason";
pub const UNKNOWN_RULE: &str = "allow-unknown-rule";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub at: Point,
    pub rule: &'static str,
    pub problem: String,
    pub why: &'static str,
    pub fix: String,
}

pub struct Located<'a> {
    pub path: &'a Path,
    pub finding: &'a Finding,
}

struct Allow {
    row: usize,
    rules: Vec<String>,
}

impl Finding {
    pub const fn new(at: Point, rule: &'static str, problem: String, why: &'static str) -> Self {
        Self {
            at,
            rule,
            problem,
            why,
            fix: String::new(),
        }
    }

    pub fn fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = fix.into();
        self
    }
}

impl fmt::Display for Located<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Finding {
            at,
            rule,
            problem,
            why,
            fix,
        } = self.finding;

        writeln!(
            f,
            "{}:{}:{}: {rule}: {problem}",
            self.path.display(),
            at.row + 1,
            at.column + 1
        )?;
        writeln!(f, "    Why: {why}")?;
        write!(f, "    How to fix: {fix}")
    }
}

/// Drops findings an allow directive covers, and reports directives that do
/// not say why or name a rule that does not exist.
pub fn apply_allows(source: &Source<'_>, findings: Vec<Finding>) -> Vec<Finding> {
    let mut allows = Vec::new();
    let mut kept = Vec::new();

    for comment in source.comments.iter().filter(|comment| comment.directive) {
        let text = comment.text();
        let Some(captures) = ALLOW.captures(&text) else {
            continue;
        };

        let rules = captures.get(1).map_or("", |group| group.as_str());
        let reason = captures.get(2).map_or("", |group| group.as_str()).trim();

        if reason.is_empty() {
            kept.push(
                Finding::new(
                    comment.start(),
                    WITHOUT_REASON,
                    "`toolbox-lint: allow(…)` without a reason.".to_owned(),
                    "An exception nobody can explain is soon a habit.",
                )
                .fix("Add the reason after the closing parenthesis, in one sentence."),
            );
        }

        let rules: Vec<String> = rules
            .split(',')
            .map(|rule| rule.trim().to_owned())
            .collect();

        for unknown in rules.iter().filter(|rule| !NAMES.contains(&rule.as_str())) {
            kept.push(
                Finding::new(
                    comment.start(),
                    UNKNOWN_RULE,
                    format!("`toolbox-lint: allow({unknown})` names no rule."),
                    "A misspelt rule suppresses nothing, and the finding it was meant for still fails.",
                )
                .fix(format!("Use one of: {}.", NAMES.join(", "))),
            );
        }

        allows.push(Allow {
            row: comment.end.row,
            rules,
        });
    }

    kept.extend(findings.into_iter().filter(|finding| {
        !allows.iter().any(|allow| {
            (allow.row == finding.at.row || allow.row + 1 == finding.at.row)
                && allow.rules.iter().any(|rule| rule == finding.rule)
        })
    }));
    kept.sort_by_key(|finding| (finding.at.row, finding.at.column));
    kept
}
