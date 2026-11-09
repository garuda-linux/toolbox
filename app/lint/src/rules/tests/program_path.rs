use super::count_at;

const RULE: &str = "program-path";
const PRODUCTION: &str = "crates/system/src/probe.rs";

#[test]
fn a_bare_program_name_is_reported() {
    let text = "fn f() {\nlet c = Command::new(\"pacman\");\n}\n";
    assert_eq!(count_at(RULE, PRODUCTION, text), 1);
}

#[test]
fn fixed_paths_variables_and_programs_pass() {
    for call in [
        "Command::new(\"/usr/bin/pacman\")",
        "Command::new(program)",
        "Command::new(Program::Pacman)",
        "Command::new(env!(\"CARGO_BIN_EXE_x\"))",
    ] {
        let text = format!("fn f() {{\nlet c = {call};\n}}\n");

        assert_eq!(count_at(RULE, PRODUCTION, &text), 0, "{call}");
    }
}

#[test]
fn a_listed_exception_passes_for_its_program_alone() {
    let opener = "fn f() {\nlet c = Command::new(\"xdg-open\");\n}\n";
    assert_eq!(count_at(RULE, "app/gui/src/links.rs", opener), 0);
    assert_eq!(count_at(RULE, PRODUCTION, opener), 1);

    let other = "fn f() {\nlet c = Command::new(\"pacman\");\n}\n";
    assert_eq!(count_at(RULE, "app/gui/src/links.rs", other), 1);
}

#[test]
fn test_files_pass() {
    let text = "fn f() {\nlet c = Command::new(\"getent\");\n}\n";
    assert_eq!(count_at(RULE, "app/helper/tests/arch/users.rs", text), 0);
}
