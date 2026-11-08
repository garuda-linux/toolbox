//! The desktop the session names, which the toggles that fit one desktop
//! only are matched against.

/// The names in `XDG_CURRENT_DESKTOP`'s value `current`, which colons
/// separate.
pub fn desktops(current: &str) -> Vec<String> {
    current
        .split(':')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}
