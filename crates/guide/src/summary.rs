//! mdBook's `SUMMARY.md`: the order of the pages and the parts that group
//! them, so the book and the app list the pages the same way.

#[cfg(test)]
mod tests;

/// A group of pages under one part heading. The pages before the first
/// heading form a part without a title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    pub title: Option<String>,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub title: String,
    /// The page's file, relative to `docs/src`.
    pub file: String,
}

/// The parts of `summary`, in order. The first heading is the summary's
/// own title, every later one starts a part. Nesting is flattened, as the
/// app lists one level.
pub fn parse(summary: &str) -> Vec<Part> {
    let mut parts = vec![Part {
        title: None,
        entries: Vec::new(),
    }];

    let mut titled = false;

    for line in summary.lines().map(str::trim) {
        if let Some(heading) = line.strip_prefix("# ") {
            if titled {
                parts.push(Part {
                    title: Some(heading.trim().to_owned()),
                    entries: Vec::new(),
                });
            }

            titled = true;
            continue;
        }

        if let Some(entry) = entry(line)
            && let Some(part) = parts.last_mut()
        {
            part.entries.push(entry);
        }
    }

    parts.retain(|part| !part.entries.is_empty());
    parts
}

/// The link of a line such as `- [Kernels](kernels.md)`.
fn entry(line: &str) -> Option<Entry> {
    let line = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .unwrap_or(line);

    let rest = line.strip_prefix('[')?;
    let (title, rest) = rest.split_once("](")?;
    let file = rest.strip_suffix(')')?;

    let file = file.strip_prefix("./").unwrap_or(file);

    (!file.is_empty()).then(|| Entry {
        title: title.to_owned(),
        file: file.to_owned(),
    })
}
