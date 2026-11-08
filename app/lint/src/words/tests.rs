use std::collections::HashSet;

use super::{identifier_words, mostly_covered, prose_words};

#[test]
fn identifiers_split_into_concepts() {
    assert_eq!(
        identifier_words("getInvoiceLines"),
        ["get", "invoice", "line"]
    );
    assert_eq!(identifier_words("MAX_PAGE_SIZE"), ["max", "page", "size"]);
    assert_eq!(identifier_words("HTTPServer"), ["http", "server"]);
    assert_eq!(identifier_words("load_cfg"), ["get", "configuration"]);
}

#[test]
fn prose_drops_stopwords_and_folds_verbs() {
    assert_eq!(
        prose_words("Fetches the lines of a file"),
        ["get", "line", "file"]
    );
}

#[test]
fn coverage_needs_three_quarters() {
    let known: HashSet<String> = ["get", "line", "file"].map(str::to_owned).into();
    assert!(mostly_covered(&prose_words("Read the file lines"), &known));
    assert!(!mostly_covered(
        &prose_words("Read the lines lazily, in pages"),
        &known
    ));
    assert!(!mostly_covered(&[], &known));
}
