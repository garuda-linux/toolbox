//! A size in bytes as people read it, the same in the window and the
//! terminal.

#[cfg(test)]
mod tests;

/// The units of a size, each 1024 times the last.
const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

/// `bytes` with one decimal in the largest unit below it: `200.0 GiB`.
pub fn size(bytes: u64) -> String {
    let mut unit = 0;
    let mut divisor: u128 = 1;
    while u128::from(bytes) >= divisor * 1024 && unit + 1 < UNITS.len() {
        divisor *= 1024;
        unit += 1;
    }

    let name = UNITS.get(unit).copied().unwrap_or_default();
    if unit == 0 {
        return format!("{bytes} {name}");
    }

    let tenths = (u128::from(bytes) * 10 + divisor / 2) / divisor;
    format!("{}.{} {name}", tenths / 10, tenths % 10)
}
