//! How long a step took, short enough for the end of its line.

#[cfg(test)]
mod tests;

use std::time::Duration;

/// `took` as `4.2 s`, `1 min 3 s` or `1 h 5 min`. Each form rounds to its
/// own unit, so a step of 59.96 seconds reads `1 min`, not `60.0 s`.
pub fn elapsed(took: Duration) -> String {
    let tenths = rounded(took, 100);
    if tenths < 600 {
        return format!("{}.{} s", tenths / 10, tenths % 10);
    }

    let seconds = rounded(took, 1_000);
    if seconds < 3_600 {
        return pair(seconds / 60, "min", seconds % 60, "s");
    }

    let minutes = rounded(took, 60_000);
    pair(minutes / 60, "h", minutes % 60, "min")
}

/// `took` in units of `unit` milliseconds, to the nearest.
const fn rounded(took: Duration, unit: u128) -> u128 {
    (took.as_millis() + unit / 2) / unit
}

/// `big` and `small` in their units, a small part of nought left out.
fn pair(big: u128, big_unit: &str, small: u128, small_unit: &str) -> String {
    if small == 0 {
        format!("{big} {big_unit}")
    } else {
        format!("{big} {big_unit} {small} {small_unit}")
    }
}
