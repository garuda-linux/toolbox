//! The INI dialect of KDE's config files: `[Group]` headers, `key=value`
//! lines, `#` comments. Nested headers (`[Colors:Window][Inactive]`) and
//! localised keys (`Name[de]`) are kept apart from the plain ones.

pub fn entries(text: &str) -> impl Iterator<Item = (&str, &str, &str)> {
    let mut group = "";
    text.lines().filter_map(move |line| {
        let line = line.trim();
        if let Some(header) = line.strip_prefix('[') {
            group = header.strip_suffix(']').unwrap_or(header);
            return None;
        }

        if line.starts_with('#') {
            return None;
        }

        let (key, value) = line.split_once('=')?;
        Some((group, key.trim(), value.trim()))
    })
}
