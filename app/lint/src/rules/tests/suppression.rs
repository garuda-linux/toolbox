use super::{count, rules};

#[test]
fn allow_directives_need_a_reason_and_cover_the_next_line() {
    let allowed =
        "// toolbox-lint: allow(no-comment-semicolon) quoting a log line\n// a; b\nfn f() {}\n";

    assert_eq!(rules(allowed), Vec::<&str>::new());

    let bare = "// toolbox-lint: allow(no-comment-semicolon)\n// a; b\nfn f() {}\n";
    assert_eq!(rules(bare), vec!["allow-without-reason"]);
    assert_eq!(count("no-comment-semicolon", "// a; b\nfn f() {}\n"), 1);
}

#[test]
fn an_allow_naming_no_rule_is_reported_and_suppresses_nothing() {
    let typo =
        "// toolbox-lint: allow(no-comment-semicolons) quoting a log line\n// a; b\nfn f() {}\n";

    assert_eq!(
        rules(typo),
        vec!["allow-unknown-rule", "no-comment-semicolon"]
    );
}

#[test]
fn every_rule_a_finding_can_have_is_a_known_name() {
    for name in super::super::NAMES {
        let text = format!("// toolbox-lint: allow({name}) a reason\nfn f() {{}}\n");

        assert_eq!(rules(&text), Vec::<&str>::new(), "{name}");
    }
}
