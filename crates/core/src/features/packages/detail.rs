#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;

use super::graph::dependency_name;
use super::list::Catalog;

pub const NAMES_SHOWN: usize = 12;

pub const HISTORY_SHOWN: usize = 5;

#[derive(Debug, PartialEq, Eq)]
pub struct Shown<'a, T> {
    pub items: &'a [T],
    pub hidden: usize,
}

/// The package a dependency string leads to: the package of its name,
/// else the installed package that provides it, as libalpm looks them
/// up. `None` when neither is known.
pub fn target(catalog: &Catalog, dependency: &str) -> Option<PackageName> {
    let wanted = dependency_name(dependency);
    let named = PackageName::new(wanted).ok()?;
    if catalog.entry(&named).is_some() {
        return Some(named);
    }

    catalog
        .local()
        .iter()
        .find(|node| {
            node.provides
                .iter()
                .any(|provided| dependency_name(provided) == wanted)
        })
        .map(|node| node.name.clone())
}

pub fn shown<T>(items: &[T], limit: usize, all: bool) -> Shown<'_, T> {
    let count = if all {
        items.len()
    } else {
        limit.min(items.len())
    };

    let (first, rest) = items.split_at(count);

    Shown {
        items: first,
        hidden: rest.len(),
    }
}

pub fn file_list(files: &[String]) -> String {
    files.iter().fold(String::new(), |mut text, file| {
        text.push_str(file);
        text.push('\n');
        text
    })
}

/// A web link as the detail page shows it, without `https://` and a
/// closing slash. `None` for anything that is no web link, which then
/// shows as plain text and never opens.
pub fn web_address(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;

    let shown = rest.strip_suffix('/').unwrap_or(rest);
    (!shown.is_empty()).then(|| shown.to_owned())
}
