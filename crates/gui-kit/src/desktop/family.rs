#[cfg(test)]
mod tests;

/// Desktops whose look is in `kdeglobals`.
const KDE: [&str; 2] = ["kde", "lxqt"];
/// Desktops whose look is in GTK's files.
const GTK: [&str; 5] = ["gnome", "x-cinnamon", "cinnamon", "xfce", "budgie"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// `kdeglobals`.
    Kde,
    /// `gtk.css` and `settings.ini`.
    Gtk,
    /// The portal alone, over the fallback palette.
    Other,
}

impl Family {
    /// The value is a colon separated list, most specific first, such as
    /// `ubuntu:GNOME`. The first known name decides.
    pub fn from_current_desktop(value: &str) -> Self {
        value
            .split(':')
            .map(str::to_ascii_lowercase)
            .find_map(|name| {
                if KDE.contains(&name.as_str()) {
                    Some(Self::Kde)
                } else if GTK.contains(&name.as_str()) {
                    Some(Self::Gtk)
                } else {
                    None
                }
            })
            .unwrap_or(Self::Other)
    }
}
