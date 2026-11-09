use std::collections::BTreeSet;

use toolbox_plan::{ConfigReload, GroupName, PackageName, UnitName, text};

use super::{Check, RawCheck, RawEntry, Requires, ToggleEntry, conflicts, extras, user_config};
use crate::{Problem, source};

impl RawEntry {
    pub(super) fn validate(
        self,
        seen: &mut BTreeSet<String>,
    ) -> Result<ToggleEntry, (String, Problem)> {
        let id = self.id.clone();

        self.build(seen).map_err(|problem| (id, problem))
    }

    fn build(self, seen: &mut BTreeSet<String>) -> Result<ToggleEntry, Problem> {
        let Self {
            id,
            title,
            description,
            check,
            also,
            conflicts,
            requires,
            restart_user_units,
            reload,
            desktop,
            icon,
            url,
        } = self;

        source::claim_id(&id, seen)?;
        texts(&title, &description)?;

        let check = check.validate()?;
        let edits_config = matches!(check, Check::UserConfig(_));

        Ok(ToggleEntry {
            also: extras::also(also, &check)?,
            conflicts: conflicts::list(conflicts)?,
            restart: user_config::restart_units(restart_user_units, edits_config)?,
            reload: user_config::reload(reload.as_deref(), edits_config)?,
            desktops: extras::desktops(desktop)?,
            icon: extras::icon(icon)?,
            url: extras::url(url)?,
            id,
            title,
            description,
            check,
            requires,
        })
    }
}

impl ToggleEntry {
    /// The stable id, unique in its catalog file.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The English title, translated in the UI.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The English description, translated in the UI.
    pub fn description(&self) -> &str {
        &self.description
    }

    pub const fn check(&self) -> &Check {
        &self.check
    }

    /// Packages installed and removed together with the package of a
    /// package check. Empty for other checks.
    pub fn also(&self) -> &[PackageName] {
        &self.also.packages
    }

    /// System units enabled once the entry's packages are installed and
    /// disabled when it is turned off. Empty for other checks.
    pub fn also_units(&self) -> &[UnitName] {
        &self.also.units
    }

    /// Units of the user's own service manager, switched like
    /// [`ToggleEntry::also_units`].
    pub fn also_user_units(&self) -> &[UnitName] {
        &self.also.user_units
    }

    /// Ids of the entries turned off when this one is turned on, for
    /// example the other `LibreOffice` branch. Each is another entry of the
    /// same file.
    pub fn conflicts(&self) -> &[String] {
        &self.conflicts
    }

    /// The entry's picture: a file name in [`ToggleCatalog::icon_dir`].
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// The website of what the entry installs: an `https` URL with a host,
    /// no blanks or control characters, at most 2048 bytes.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Ids of the entries this one depends on. Each is another entry of the
    /// same file and following them never leads back here.
    #[must_use = "the requirements decide whether the entry can be changed"]
    pub const fn requires(&self) -> &Requires {
        &self.requires
    }

    /// User units restarted after the entry's config edit, so the program
    /// reads the changed file. Empty for other checks.
    pub fn restart_user_units(&self) -> &[UnitName] {
        &self.restart
    }

    pub const fn reload(&self) -> Option<ConfigReload> {
        self.reload
    }

    /// Whether the entry belongs on a session whose `XDG_CURRENT_DESKTOP`
    /// lists `current`. Desktop names are compared without case.
    pub fn is_for_desktop(&self, current: &[String]) -> bool {
        self.desktops.is_empty()
            || self.desktops.iter().any(|wanted| {
                current
                    .iter()
                    .any(|desktop| desktop.eq_ignore_ascii_case(wanted))
            })
    }
}

impl RawCheck {
    pub(super) fn validate(self) -> Result<Check, Problem> {
        let name = Problem::BadName;

        Ok(match self {
            Self::Package(value) => Check::Package(PackageName::new(value).map_err(name)?),
            Self::Service(value) => Check::Service(UnitName::new(value).map_err(name)?),
            Self::UserService(value) => Check::UserService(UnitName::new(value).map_err(name)?),
            Self::Group(value) => {
                let group = GroupName::new(value).map_err(name)?;

                if group.is_admin() {
                    return Err(Problem::AdminGroup(group.as_str().to_owned()));
                }

                Check::Group(group)
            }
            Self::UserFile(file) => Check::UserConfig(file.validate()?),
            Self::UserIni(ini) => Check::UserConfig(ini.validate()?),
            Self::UserJson(json) => Check::UserConfig(json.validate()?),
        })
    }
}

fn texts(title: &str, description: &str) -> Result<(), Problem> {
    if source::is_blank(title) {
        return Err(Problem::EmptyTitle);
    }

    if source::is_blank(description) {
        return Err(Problem::EmptyDescription);
    }

    // A text may run over lines, nothing else hidden passes.
    [title, description]
        .into_iter()
        .find(|shown| shown.chars().any(|ch| ch != '\n' && text::is_hidden(ch)))
        .map_or(Ok(()), |shown| {
            Err(Problem::HiddenCharacter(shown.to_owned()))
        })
}
