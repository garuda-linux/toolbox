//! The one rule every checked path shares.

/// Whether every `/`-separated part of `path` is neither empty, `.` nor
/// `..`, so the path names one place and never climbs out of its root.
pub fn has_plain_components(path: &str) -> bool {
    path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}
