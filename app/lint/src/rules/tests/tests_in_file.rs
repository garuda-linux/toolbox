use super::count_at;

const RULE: &str = "tests-in-file";
const INLINE: &str = "#[cfg(test)]\nmod tests {\n#[test]\nfn t() {}\n}\n";

#[test]
fn an_inline_tests_module_in_src_is_reported() {
    assert_eq!(count_at(RULE, "crates/plan/src/name.rs", INLINE), 1);
}

#[test]
fn a_tests_file_and_other_modules_pass() {
    let declared = "#[cfg(test)]\nmod tests;\n";
    assert_eq!(count_at(RULE, "crates/plan/src/name.rs", declared), 0);

    let other = "mod inner {\nfn f() {}\n}\n";
    assert_eq!(count_at(RULE, "crates/plan/src/name.rs", other), 0);
}

#[test]
fn files_outside_src_pass() {
    assert_eq!(count_at(RULE, "app/helper/tests/arch.rs", INLINE), 0);
}
