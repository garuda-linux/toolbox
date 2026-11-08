use regex::Regex;

/// Every pattern is a constant that the rule tests exercise, so a broken one
/// fails `cargo test` long before it could fail a lint run.
#[expect(
    clippy::expect_used,
    reason = "patterns are constants exercised by the tests"
)]
pub fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid pattern")
}
