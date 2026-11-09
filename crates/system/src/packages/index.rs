//! The Packages page's reads through libalpm: the index of every package
//! and the details of one.

use std::collections::BTreeMap;

use alpm::{AlpmList, Dep, PackageReason, Pkg, SigLevel};
use toolbox_plan::PackageName;
use toolbox_state::packages::{
    IndexEntry, InstallReason, Installed, InstalledDetails, LocalNode, OptionalDependency, Origin,
    PackageDetails, PackageIndex,
};

use super::{Libalpm, failed, open_local};
use crate::error::ProbeError;

pub trait PackageIndexSource: Send + Sync {
    /// Every package the sync databases offer and every installed one,
    /// sorted by name, with the installed packages' dependency graph.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager or without
    /// any synced database, [`ProbeError::Failed`] when one cannot be read.
    fn index(&self) -> Result<PackageIndex, ProbeError>;

    /// One package, from the local database when it is installed, else
    /// from the first repository that offers it. `None` when neither has
    /// it.
    ///
    /// # Errors
    ///
    /// As [`PackageIndexSource::index`].
    fn details(&self, package: &PackageName) -> Result<Option<PackageDetails>, ProbeError>;
}

/// What the index keeps of an installed package until it is matched
/// against the offered ones.
struct LocalFacts {
    installed: Installed,
    description: String,
    size: u64,
}

impl PackageIndexSource for Libalpm {
    fn index(&self) -> Result<PackageIndex, ProbeError> {
        let conf = self.conf()?;
        let handle = open_local(&conf)?;
        let sync = self.sync.read(&conf)?;
        let mut local = Vec::new();
        let mut facts = BTreeMap::new();

        for package in handle.localdb().pkgs() {
            let Some(node) = local_node(package) else {
                continue;
            };

            let name = node.name.clone();
            local.push(node);
            facts.insert(
                name,
                LocalFacts {
                    installed: Installed {
                        version: package.version().to_string(),
                        reason: reason(package.reason()),
                    },
                    description: package.desc().unwrap_or_default().to_owned(),
                    size: size(package),
                },
            );
        }

        let mut entries: Vec<IndexEntry> = sync
            .entries
            .iter()
            .map(|offered| {
                let mut entry = offered.clone();
                if let Some(found) = facts.remove(&entry.name) {
                    entry.size = found.size;
                    entry.installed = Some(found.installed);
                }

                entry
            })
            .collect();

        entries.extend(facts.into_iter().map(|(name, found)| IndexEntry {
            name,
            description: found.description,
            version: found.installed.version.clone(),
            origin: Origin::Foreign,
            size: found.size,
            installed: Some(found.installed),
        }));
        entries.sort_by(|left, right| left.name.cmp(&right.name));

        Ok(PackageIndex { entries, local })
    }

    /// The cached sync read names the repository and version, so only a
    /// package that is not installed opens one sync database, its own.
    fn details(&self, package: &PackageName) -> Result<Option<PackageDetails>, ProbeError> {
        let conf = self.conf()?;
        let handle = open_local(&conf)?;
        let sync = match self.sync.read(&conf) {
            Ok(sync) => Some(sync),
            Err(ProbeError::Unavailable) => None,
            Err(error) => return Err(error),
        };

        let offered = sync
            .as_ref()
            .and_then(|sync| sync.entries.iter().find(|entry| entry.name == *package));

        let name = package.as_str();
        let local = handle.localdb().pkg(name).ok();

        let shown: &Pkg = match (local, offered.map(|entry| &entry.origin)) {
            (Some(installed), _) => installed,
            (None, Some(Origin::Repository(repo))) => {
                let db = handle
                    .register_syncdb(repo.as_str(), SigLevel::NONE)
                    .map_err(failed(repo))?;

                db.pkg(name).map_err(failed(repo))?
            }
            (None, _) => return Ok(None),
        };

        let (origin, version) = offered.map_or_else(
            || (Origin::Foreign, shown.version().to_string()),
            |entry| (entry.origin.clone(), entry.version.clone()),
        );

        Ok(Some(PackageDetails {
            name: package.clone(),
            description: shown.desc().unwrap_or_default().to_owned(),
            version,
            origin,
            size: size(shown),
            url: shown.url().map(str::to_owned),
            licenses: shown.licenses().iter().map(str::to_owned).collect(),
            groups: shown.groups().iter().map(str::to_owned).collect(),
            depends: strings(shown.depends()),
            optional: shown
                .optdepends()
                .iter()
                .map(|dep| OptionalDependency {
                    name: dep.name().to_owned(),
                    reason: dep.desc().unwrap_or_default().to_owned(),
                })
                .collect(),
            required_by: local
                .map(|installed| installed.required_by().iter().map(str::to_owned).collect())
                .unwrap_or_default(),
            provides: strings(shown.provides()),
            conflicts: strings(shown.conflicts()),
            replaces: strings(shown.replaces()),
            installed: local.map(|installed| installed_details(installed)),
        }))
    }
}

fn installed_details(package: &Pkg) -> InstalledDetails {
    InstalledDetails {
        version: package.version().to_string(),
        reason: reason(package.reason()),
        date: package.install_date(),
        files: package
            .files()
            .files()
            .iter()
            .map(|file| format!("/{}", String::from_utf8_lossy(file.name())))
            .collect(),
    }
}

/// `None` for a name that is no `PackageName`, which no removal can name.
pub(super) fn local_node(package: &Pkg) -> Option<LocalNode> {
    Some(LocalNode {
        name: PackageName::new(package.name()).ok()?,
        depends: strings(package.depends()),
        provides: strings(package.provides()),
        reason: reason(package.reason()),
    })
}

fn strings(dependencies: AlpmList<&Dep>) -> Vec<String> {
    dependencies.iter().map(ToString::to_string).collect()
}

const fn reason(reason: PackageReason) -> InstallReason {
    match reason {
        PackageReason::Explicit => InstallReason::Explicit,
        PackageReason::Depend => InstallReason::Dependency,
    }
}

fn size(package: &Pkg) -> u64 {
    u64::try_from(package.isize()).unwrap_or_default()
}
