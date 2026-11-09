use super::count_at;

const RULE: &str = "no-shell-command";
const PRODUCTION: &str = "crates/exec/src/run.rs";

#[test]
fn shells_by_name_or_path_are_reported() {
    for program in ["sh", "bash", "/bin/sh", "/usr/bin/bash"] {
        let text = format!("fn f() {{\nlet c = std::process::Command::new(\"{program}\");\n}}\n");

        assert_eq!(count_at(RULE, PRODUCTION, &text), 1, "{program}");
    }
}

#[test]
fn pty_commands_and_raw_strings_are_read_too() {
    let text = "fn f() {\nlet c = PtyCommand::new(r\"/bin/sh\");\n}\n";
    assert_eq!(count_at(RULE, PRODUCTION, text), 1);
}

#[test]
fn other_programs_and_test_files_pass() {
    let program = "fn f() {\nlet c = Command::new(\"/usr/bin/shred\");\n}\n";
    assert_eq!(count_at(RULE, PRODUCTION, program), 0);

    let shell = "fn f() {\nlet c = Command::new(\"/bin/sh\");\n}\n";
    for test in [
        "crates/exec/src/run/tests.rs",
        "crates/exec/src/run/tests/fake.rs",
        "crates/pty/tests/session.rs",
    ] {
        assert_eq!(count_at(RULE, test, shell), 0, "{test}");
    }
}
