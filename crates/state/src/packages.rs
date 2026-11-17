//! Repository and installed packages as the Packages page lists them: a
//! light index of every package, and the details of one.

use toolbox_plan::PackageName;
use toolbox_text::OutsideText;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Origin {
    /// A sync database of `pacman.conf`, by its section name.
    Repository(String),
    /// Installed, but no configured repository offers it.
    Foreign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstallReason {
    /// Asked for by name.
    Explicit,
    /// Pulled in by another package.
    Dependency,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installed {
    pub version: String,
    pub reason: InstallReason,
}

/// One row of the index: a package a repository offers, an installed one
/// or both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexEntry {
    pub name: PackageName,
    pub description: OutsideText,
    /// The repository's version, the installed one for a foreign package.
    pub version: String,
    pub origin: Origin,
    /// Bytes on disk once installed: the installed build's when there is
    /// one, else the repository's.
    pub size: u64,
    pub installed: Option<Installed>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalNode {
    pub name: PackageName,
    /// Dependency strings as pacman writes them (`glibc>=2.40`, `sh`).
    pub depends: Vec<String>,
    /// What it stands in for, in the same form.
    pub provides: Vec<String>,
    /// The names of its optional dependencies, without what each is for.
    pub optional: Vec<String>,
    pub reason: InstallReason,
}

/// Every package of the sync databases and every installed one, by
/// name, plus the installed packages' dependency graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageIndex {
    pub entries: Vec<IndexEntry>,
    pub local: Vec<LocalNode>,
}

/// A dependency a package works without, and what it adds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionalDependency {
    pub name: PackageName,
    pub reason: OutsideText,
}

/// The installed side of [`PackageDetails`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledDetails {
    pub version: String,
    pub reason: InstallReason,
    /// Seconds since the epoch.
    pub date: Option<i64>,
    /// Absolute paths, folders ending in `/`.
    pub files: Vec<OutsideText>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageDetails {
    pub name: PackageName,
    pub description: OutsideText,
    pub version: String,
    pub origin: Origin,
    pub size: u64,
    pub url: Option<OutsideText>,
    pub licenses: Vec<OutsideText>,
    pub groups: Vec<OutsideText>,
    pub depends: Vec<String>,
    pub optional: Vec<OptionalDependency>,
    /// The installed packages that depend on it.
    pub required_by: Vec<OutsideText>,
    pub provides: Vec<String>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,
    pub installed: Option<InstalledDetails>,
}

/// The installed packages' dependency graph, for the guards that ask what
/// a `pacman -Rns` takes along.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageGraph(pub Vec<LocalNode>);

impl crate::Part for PackageGraph {}
