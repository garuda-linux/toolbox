use toolbox_core::{Summary, SummaryCode};

use super::{sentence, sentence_in};
use crate::i18n;

#[test]
fn a_single_value_fills_the_name() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(&ctx, &Summary::of(SummaryCode::PackageInstall, "firefox")),
        "Install firefox"
    );
}

#[test]
fn two_values_fill_their_own_names() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(
            &ctx,
            &Summary::pair(SummaryCode::GroupJoin, "nico", "wheel")
        ),
        "Add nico to the group wheel"
    );
}

#[test]
fn numbers_pick_the_plural_form() {
    let ctx = egui::Context::default();

    assert_eq!(
        sentence(&ctx, &Summary::of(SummaryCode::GrubTimeout, "1")),
        "Show the boot menu for 1 second"
    );
    assert_eq!(
        sentence(&ctx, &Summary::of(SummaryCode::GrubTimeout, "5")),
        "Show the boot menu for 5 seconds"
    );
}

/// The command line has no window, so it names the language itself.
#[test]
fn a_language_named_directly_words_the_same_change() {
    let ctx = egui::Context::default();
    let german: fluent_templates::LanguageIdentifier = "de-DE".parse().unwrap();
    i18n::set_language(&ctx, &german);

    let summary = Summary::pair(SummaryCode::GroupJoin, "nico", "wheel");

    assert_eq!(sentence_in(&german, &summary), sentence(&ctx, &summary));
}
