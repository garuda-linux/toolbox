use super::count;

const RULE: &str = "no-restating-comment";

#[test]
fn restating_comments_are_reported() {
    for code in [
        "// Set loading to true\nself.loading = true;",
        "// Get the user from the session\nlet user = session.user;",
        "// Loop over all lines\nfor line in lines {}",
        "// Check if the user is an admin\nif user.is_admin {}",
        "// Load the lock file\nlet lock = load_lock_file();",
    ] {
        assert_eq!(
            count(RULE, &format!("fn f() {{\n{code}\n}}\n")),
            1,
            "{code}"
        );
    }
}

#[test]
fn trailing_restatements_are_reported() {
    assert_eq!(
        count(
            RULE,
            "fn f() {\n    let mut count = 1;\n    count = 0; // reset count\n}\n"
        ),
        1
    );
}

#[test]
fn reasons_and_information_pass() {
    let text = "fn f() {\n// The backend recalculates the price after a PATCH\nlet price = line.price;\n// Set to Y, otherwise the server ignores the line\nflag = 'Y';\n}\n";
    assert_eq!(count(RULE, text), 0);
}
