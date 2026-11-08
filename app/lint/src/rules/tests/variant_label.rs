use super::count;

const RULE: &str = "no-variant-label";

#[test]
fn backticked_variants_are_reported() {
    let text = "fn f() {\nmatch c {\n// Erase in line (`K`, `1K`): redrawn from its start.\n'K' => {},\n}\n}\n";
    assert_eq!(count(RULE, text), 1);
}

#[test]
fn reasons_and_outside_functions_pass() {
    let text = "fn f() {\nmatch c {\n// `K` must clear because the row is reused by nom.\n'K' => {},\n}\n}\n";
    assert_eq!(count(RULE, text), 0);

    let constant = "const C: u8 = match 'K' {\n// Erase in line (`K`).\n'K' => 1,\n_ => 0,\n};\n";
    assert_eq!(count(RULE, constant), 0);

    let inside =
        "fn f() {\nlet c = match 'K' {\n// Erase in line (`K`).\n'K' => 1,\n_ => 0,\n};\n}\n";

    assert_eq!(count(RULE, inside), 1);
}
