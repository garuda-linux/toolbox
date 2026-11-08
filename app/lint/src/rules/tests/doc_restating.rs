use super::count;

const RULE: &str = "no-restating-doc";

#[test]
fn docs_that_repeat_the_name_are_reported() {
    assert_eq!(
        count(
            RULE,
            "/// Gets the invoice lines.\nfn get_invoice_lines() {}\n"
        ),
        1
    );
    assert_eq!(
        count(
            RULE,
            "/// The mirror list.\n#[must_use]\npub struct MirrorList;\n"
        ),
        1
    );
}

#[test]
fn docs_with_information_pass() {
    let text = "/// Lines of the invoice, sorted by position. Empty for drafts.\nfn get_invoice_lines() {}\n";
    assert_eq!(count(RULE, text), 0);
}
