//! `busctl --json=short call ... ListUnits` and `ListUnitFiles` in the Lima VM.

/// `busctl --json=short call ... ListUnits`, a `{"type", "data"}` envelope.
pub const LIST_UNITS_JSON: &str = include_str!("../systemd/fixtures/list-units.json");

/// `busctl --json=short call ... ListUnitFiles`, a `{"type", "data"}` envelope.
pub const LIST_UNIT_FILES_JSON: &str = include_str!("../systemd/fixtures/list-unit-files.json");
