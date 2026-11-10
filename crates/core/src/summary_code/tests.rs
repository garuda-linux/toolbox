use std::collections::BTreeSet;

use strum::IntoEnumIterator as _;

use super::SummaryCode;

/// The id is a Fluent id's tail, so two codes never share one.
#[test]
fn every_code_has_its_own_kebab_case_id() {
    let ids: Vec<&str> = SummaryCode::iter().map(SummaryCode::as_str).collect();
    let unique: BTreeSet<&str> = ids.iter().copied().collect();

    assert_eq!(unique.len(), ids.len());
    for id in ids {
        assert!(
            id.chars()
                .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-'),
            "{id}"
        );
    }
}

#[test]
fn removals_masks_and_blacklists_show_before_they_apply() {
    let destructive: Vec<&str> = SummaryCode::iter()
        .filter(|code| code.destructive())
        .map(SummaryCode::as_str)
        .collect();

    assert_eq!(
        destructive,
        [
            "driver-blacklist",
            "group-leave",
            "group-remove",
            "package-remove",
            "unit-mask",
            "user-remove-keep-home",
            "user-remove-with-home",
            "user-unit-mask",
        ]
    );
}
