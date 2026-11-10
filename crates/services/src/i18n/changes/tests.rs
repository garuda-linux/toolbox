use strum::IntoEnumIterator as _;
use toolbox_core::{Summary, SummaryCode};

use super::sentence_in;
use crate::i18n::available;

/// Every code the core can name has its sentence, so no change shows as
/// its bare Fluent id.
#[test]
fn every_change_code_has_a_sentence_in_every_language() {
    for language in available() {
        for code in SummaryCode::iter() {
            let summary = Summary::pair(code, "first", "second");
            let id = format!("change-{}", code.as_str());
            assert_ne!(sentence_in(&language, &summary), id, "{id} in {language}");
        }
    }
}
