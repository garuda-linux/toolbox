use std::rc::Rc;

use super::found::{Pick, Row, Section, cached};
use crate::testing::PageFixture;

#[test]
fn the_same_text_on_the_same_queue_keeps_the_found_rows() {
    let ctx = egui::Context::default();
    let fixture = PageFixture::default();
    let mut found = None;

    let first = cached(&mut found, &ctx, &fixture.context(), "ser", &[]);
    let again = cached(&mut found, &ctx, &fixture.context(), "ser", &[]);

    assert!(Rc::ptr_eq(&first, &again));
}

#[test]
fn a_new_text_queue_or_language_finds_the_rows_again() {
    let ctx = egui::Context::default();
    let mut fixture = PageFixture::default();
    let mut found = None;
    let first = cached(&mut found, &ctx, &fixture.context(), "ser", &[]);

    let typed = cached(&mut found, &ctx, &fixture.context(), "use", &[]);
    assert!(!Rc::ptr_eq(&first, &typed), "a new text kept the rows");

    fixture.revision += 1;
    let queued = cached(&mut found, &ctx, &fixture.context(), "use", &[]);
    assert!(!Rc::ptr_eq(&typed, &queued), "a new queue kept the rows");

    crate::i18n::set_language(&ctx, &"de-DE".parse().expect("a language"));
    let german = cached(&mut found, &ctx, &fixture.context(), "use", &[]);
    assert!(
        !Rc::ptr_eq(&queued, &german),
        "a new language kept the rows"
    );
}

fn maintenance(rows: &[Row]) -> Vec<&Row> {
    rows.iter()
        .filter(|row| row.section == Section::Maintenance)
        .collect()
}

#[test]
fn a_section_without_a_page_shows_five_rows_and_expands_in_place() {
    let ctx = egui::Context::default();
    let fixture = PageFixture::default();
    let capped = cached(&mut None, &ctx, &fixture.context(), "e", &[]);
    let shown = maintenance(&capped);
    let (last, best) = shown.split_last().expect("Maintenance rows");

    assert_eq!(best.len(), 5);
    assert_eq!(last.pick, Pick::Expand(Section::Maintenance));

    let expanded = cached(
        &mut None,
        &ctx,
        &fixture.context(),
        "e",
        &[Section::Maintenance],
    );

    let all = maintenance(&expanded);

    assert!(all.len() > 6, "{}", all.len());
    assert!(
        all.iter()
            .all(|row| row.pick != Pick::Expand(Section::Maintenance))
    );
}
