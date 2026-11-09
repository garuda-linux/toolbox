//! One read of the sync databases, kept until a database file changes.
//! libalpm unpacks a whole database to list it, which takes far longer
//! than anything else the snapshot probes do, and the files only change
//! with `pacman -Sy`.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use alpm::SigLevel;
use parking_lot::Mutex;
use toolbox_plan::PackageName;
use toolbox_state::kernels::RepoPackage;
use toolbox_state::packages::{IndexEntry, Origin};
use toolbox_state::parts::SyncPackages;

use super::pacman_conf::PacmanConf;
use super::{LINUX, description, failed, open};
use crate::error::ProbeError;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SyncRead {
    pub offered: SyncPackages,
    /// The packages whose name starts with `linux`, repositories in the
    /// order of `pacman.conf`, names sorted within one, as `pacman -Ss`
    /// lists them.
    pub linux: Vec<RepoPackage>,
    /// Every offered package once, from the first repository that has
    /// it, not yet matched against the installed ones.
    pub entries: Vec<IndexEntry>,
}

/// A synced database file: its repository, and what tells a new download
/// from the old one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Database {
    repo: String,
    modified: Option<SystemTime>,
    size: u64,
}

#[derive(Debug, Default)]
pub struct SyncCache {
    last: Mutex<Option<(Vec<Database>, Arc<SyncRead>)>>,
}

impl SyncCache {
    /// The packages of the databases `conf` names, read again only when a
    /// database file changed. Probes that ask at the same time wait for
    /// one read instead of starting their own.
    pub fn read(&self, conf: &PacmanConf) -> Result<Arc<SyncRead>, ProbeError> {
        let sync = conf.db_path.join("sync");
        let synced: Vec<Database> = conf
            .repos
            .iter()
            .filter_map(|repo| database(&sync, repo))
            .collect();

        if synced.is_empty() {
            return Err(ProbeError::Unavailable);
        }

        let mut last = self.last.lock();

        if let Some((_, read)) = last.as_ref().filter(|(databases, _)| *databases == synced) {
            return Ok(Arc::clone(read));
        }

        let read = Arc::new(read(conf, &synced)?);

        *last = Some((synced, Arc::clone(&read)));
        drop(last);
        Ok(read)
    }
}

fn database(sync: &Path, repo: &str) -> Option<Database> {
    let metadata = std::fs::metadata(sync.join(format!("{repo}.db"))).ok()?;

    metadata.is_file().then(|| Database {
        repo: repo.to_owned(),
        modified: metadata.modified().ok(),
        size: metadata.len(),
    })
}

fn read(conf: &PacmanConf, synced: &[Database]) -> Result<SyncRead, ProbeError> {
    let handle = open(conf)?;
    let mut offered = BTreeSet::new();
    let mut linux = Vec::new();
    let mut entries = Vec::new();

    for database in synced {
        let repo = database.repo.as_str();

        // Signatures matter when pacman installs from the database. To
        // read it, one whose key is not trusted here will do.
        let db = handle
            .register_syncdb(repo, SigLevel::NONE)
            .map_err(failed(repo))?;

        db.is_valid().map_err(failed(repo))?;

        let mut kernels = Vec::new();

        for package in db.pkgs() {
            let Ok(name) = PackageName::new(package.name()) else {
                tracing::debug!("skipped the package name {}", package.name());
                continue;
            };

            if name.as_str().starts_with(LINUX) {
                kernels.push(RepoPackage {
                    repo: repo.to_owned(),
                    name: name.clone(),
                    version: package.version().to_string(),
                    description: description(package),
                });
            }

            // pacman installs from the first repository that has a name.
            if offered.insert(name.clone()) {
                entries.push(IndexEntry {
                    name,
                    description: description(package),
                    version: package.version().to_string(),
                    origin: Origin::Repository(repo.to_owned()),
                    size: u64::try_from(package.isize()).unwrap_or_default(),
                    installed: None,
                });
            }
        }

        kernels.sort_by(|left, right| left.name.cmp(&right.name));
        linux.extend(kernels);
    }

    Ok(SyncRead {
        offered: SyncPackages(offered),
        linux,
        entries,
    })
}
