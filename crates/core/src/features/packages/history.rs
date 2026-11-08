//! One package's changes from pacman's log, for the History tab.

#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;

const ALPM: &str = "] [ALPM] ";

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum HistoryAction {
    Installed,
    Upgraded,
    Downgraded,
    Reinstalled,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    /// The time stamp as pacman wrote it.
    pub when: String,
    pub action: HistoryAction,
    /// One version, or `old -> new`.
    pub versions: String,
}

impl HistoryAction {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl HistoryEntry {
    pub fn shown_when(&self) -> String {
        match self
            .when
            .get(..16)
            .and_then(|minute| minute.split_once('T'))
        {
            Some((date, time)) => format!("{date} {time}"),
            None => self.when.clone(),
        }
    }
}

/// The changes of `package` in `log`, newest first.
pub fn history(log: &str, package: &PackageName) -> Vec<HistoryEntry> {
    let mut entries: Vec<HistoryEntry> = log
        .lines()
        .filter_map(|line| entry(line, package.as_str()))
        .collect();

    entries.reverse();
    entries
}

/// `[when] [ALPM] <action> <package> (<versions>)`, for `package` alone.
fn entry(line: &str, package: &str) -> Option<HistoryEntry> {
    let (when, rest) = line.strip_prefix('[')?.split_once(ALPM)?;
    let (verb, rest) = rest.split_once(' ')?;
    let versions = rest
        .strip_prefix(package)?
        .strip_prefix(" (")?
        .strip_suffix(')')?;

    let action = match verb {
        "installed" => HistoryAction::Installed,
        "upgraded" => HistoryAction::Upgraded,
        "downgraded" => HistoryAction::Downgraded,
        "reinstalled" => HistoryAction::Reinstalled,
        "removed" => HistoryAction::Removed,
        _ => return None,
    };

    Some(HistoryEntry {
        when: when.to_owned(),
        action,
        versions: versions.to_owned(),
    })
}
