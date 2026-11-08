//! The Languages page on the real system: its locales where
//! `/etc/locale.gen` makes them, a word that it does not on other systems.
//! The kittest tests cover the clicks, a dry run here queues nothing.

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Languages"),
        Step::WaitEither(
            "System language",
            "Locales are not available on this system",
        ),
        Step::Shot("locales"),
        Step::radio("Language packs"),
        Step::WaitContaining("anguage packs"),
        Step::Shot("locales-packs"),
    ]
}
