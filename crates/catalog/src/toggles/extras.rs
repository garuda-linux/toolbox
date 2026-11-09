//! Entry fields beside the check: the packages and units that come along
//! with an entry's package, its desktops, its picture and its website.
//!
//! ```toml
//! check = { package = "gamemode" }
//! also = [{ package = "lib32-gamemode" }, { service = "gamemoded.service" }]
//! desktop = ["KDE"]
//! icon = "gamemode.png"
//! url = "https://github.com/FeralInteractive/gamemode"
//! ```

use serde::Deserialize;
use toolbox_plan::{PackageName, UnitName};

use super::Check;
use crate::{Problem, source};

const MAX_URL: usize = 2048;

/// One more thing changed together with the entry: a package installed
/// with it, or a unit enabled once the packages are there.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum RawAlso {
    Package(String),
    Service(String),
    UserService(String),
}

/// None of it decides the shown state, the check does.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Also {
    pub packages: Vec<PackageName>,
    pub units: Vec<UnitName>,
    pub user_units: Vec<UnitName>,
}

pub fn also(raw: Option<Vec<RawAlso>>, check: &Check) -> Result<Also, Problem> {
    let Some(raw) = raw else {
        return Ok(Also::default());
    };

    let Check::Package(main) = check else {
        return Err(Problem::AlsoWithoutPackage);
    };

    if raw.is_empty() {
        return Err(Problem::EmptyAlso);
    }

    let mut also = Also::default();

    for extra in raw {
        match extra {
            RawAlso::Package(name) => {
                let package = PackageName::new(name).map_err(Problem::BadName)?;

                if package == *main || also.packages.contains(&package) {
                    return Err(Problem::AlsoRepeats(package.to_string()));
                }

                also.packages.push(package);
            }
            RawAlso::Service(name) => push_unit(&mut also.units, name, Problem::AlsoRepeats)?,
            RawAlso::UserService(name) => {
                push_unit(&mut also.user_units, name, Problem::AlsoRepeats)?;
            }
        }
    }

    Ok(also)
}

pub fn push_unit(
    units: &mut Vec<UnitName>,
    name: String,
    twice: fn(String) -> Problem,
) -> Result<(), Problem> {
    let unit = UnitName::new(name).map_err(Problem::BadName)?;

    if units.contains(&unit) {
        return Err(twice(unit.to_string()));
    }

    units.push(unit);
    Ok(())
}

/// `XDG_CURRENT_DESKTOP` names. An empty list in the file would hide the
/// entry everywhere, so it is refused rather than read as "any".
pub fn desktops(names: Option<Vec<String>>) -> Result<Vec<String>, Problem> {
    let Some(names) = names else {
        return Ok(Vec::new());
    };

    if names.is_empty() || names.iter().any(|name| source::is_blank(name)) {
        return Err(Problem::EmptyDesktop);
    }

    Ok(names)
}

/// A file name in the catalog's icon folder. Plain names only, so an entry
/// can never point the app at another file.
pub fn icon(name: Option<String>) -> Result<Option<String>, Problem> {
    let Some(name) = name else {
        return Ok(None);
    };

    let plain = !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '.' | '_' | '-'));

    if plain {
        Ok(Some(name))
    } else {
        Err(Problem::BadIcon(name))
    }
}

/// The app hands this link to the browser, so only plain `https` links
/// with a host pass: no other scheme, no credentials before the host, no
/// blank or control character that could hide what is opened.
pub fn url(link: Option<String>) -> Result<Option<String>, Problem> {
    let Some(link) = link else {
        return Ok(None);
    };

    if is_web_url(&link) {
        Ok(Some(link))
    } else {
        Err(Problem::BadUrl(link))
    }
}

pub fn is_web_url(link: &str) -> bool {
    let clean = link.len() <= MAX_URL
        && !link
            .chars()
            .any(|letter| letter.is_whitespace() || letter.is_control());

    clean
        && link
            .strip_prefix("https://")
            .and_then(|rest| rest.split(['/', '?', '#']).next())
            .is_some_and(is_host)
}

/// ASCII only, so a look-alike letter of another script or a fullwidth
/// digit cannot pass for the host or port it imitates.
fn is_host(authority: &str) -> bool {
    let (host, port) = authority.split_once(':').unwrap_or((authority, ""));

    let labels_ok = host.split('.').all(|label| {
        label.starts_with(|letter: char| letter.is_ascii_alphanumeric())
            && label
                .chars()
                .all(|letter| letter.is_ascii_alphanumeric() || letter == '-')
    });

    labels_ok && is_port(port)
}

fn is_port(port: &str) -> bool {
    port.is_empty()
        || (port.len() <= 5
            && port.chars().all(|digit| digit.is_ascii_digit())
            && port.parse::<u16>().is_ok())
}
