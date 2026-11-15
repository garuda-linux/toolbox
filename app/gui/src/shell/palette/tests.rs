use std::rc::Rc;

use super::found::cached;
use crate::testing::PageFixture;

#[test]
fn the_same_text_on_the_same_queue_keeps_the_found_rows() {
    let ctx = egui::Context::default();
    let fixture = PageFixture::default();
    let mut found = None;

    let first = cached(&mut found, &ctx, &fixture.context(), "ser");
    let again = cached(&mut found, &ctx, &fixture.context(), "ser");

    assert!(Rc::ptr_eq(&first, &again));
}

#[test]
fn a_new_text_queue_or_language_finds_the_rows_again() {
    let ctx = egui::Context::default();
    let mut fixture = PageFixture::default();
    let mut found = None;
    let first = cached(&mut found, &ctx, &fixture.context(), "ser");

    let typed = cached(&mut found, &ctx, &fixture.context(), "use");
    assert!(!Rc::ptr_eq(&first, &typed), "a new text kept the rows");

    fixture.revision += 1;
    let queued = cached(&mut found, &ctx, &fixture.context(), "use");
    assert!(!Rc::ptr_eq(&typed, &queued), "a new queue kept the rows");

    crate::i18n::set_language(&ctx, &"de-DE".parse().expect("a language"));
    let german = cached(&mut found, &ctx, &fixture.context(), "use");
    assert!(
        !Rc::ptr_eq(&queued, &german),
        "a new language kept the rows"
    );
}
