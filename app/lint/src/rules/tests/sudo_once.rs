use super::count_at;

const RULE: &str = "sudo-once";
const LAUNCHER: &str = "crates/services/src/apply/launcher.rs";
const ELSEWHERE: &str = "crates/commands/src/root.rs";

#[test]
fn sudo_outside_the_launcher_is_reported() {
    for literal in ["\"sudo\"", "\"/usr/bin/sudo\"", "r\"/bin/sudo\""] {
        let text = format!("const S: &str = {literal};\n");

        assert_eq!(count_at(RULE, ELSEWHERE, &text), 1, "{literal}");
        assert_eq!(count_at(RULE, LAUNCHER, &text), 0, "{literal}");
    }
}

#[test]
fn sudo_inside_other_text_and_in_tests_passes() {
    for literal in ["\"--sudo\"", "\"sudoers\"", "\"asking through sudo\""] {
        let text = format!("const S: &str = {literal};\n");

        assert_eq!(count_at(RULE, ELSEWHERE, &text), 0, "{literal}");
    }

    let text = "const S: &str = \"sudo\";\n";
    assert_eq!(count_at(RULE, "crates/commands/src/root/tests.rs", text), 0);
}

#[test]
fn an_absolute_path_to_the_launcher_counts_as_the_launcher() {
    let text = "const S: &str = \"/usr/bin/sudo\";\n";
    let path = format!("/home/dev/toolbox/{LAUNCHER}");

    assert_eq!(count_at(RULE, &path, text), 0);
}
