use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::runtime::Runtime;
use toolbox_aur::{Answer, AurError, AurSource, Cache, Clock, RequestError, Transport, Validators};
use toolbox_cli::args::{Confirm, PackageInstall};
use toolbox_core::domains::{AurPackages, InstalledPackages};
use toolbox_core::features::packages::aur::{AurIndex, AurPackage};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::PackageName;
use toolbox_services::catalogs::Catalogs;

use super::{Sources, build};
use crate::change::Refused;
use crate::queue::add::Queueable;

#[derive(Default)]
struct Copy {
    reads: AtomicUsize,
}

impl AurSource for Copy {
    fn index(&self) -> Result<AurIndex, AurError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(AurIndex {
            packages: ["paru-bin", "yay-bin"].map(package).to_vec(),
            downloaded: 0,
        })
    }
}

#[derive(Default)]
struct Offline {
    requests: AtomicUsize,
}

impl Transport for Offline {
    fn get(&self, _url: &str, _validators: &Validators) -> Result<Answer, RequestError> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        Err(RequestError::NoAnswer("offline".to_owned()))
    }
}

struct Fixed;

impl Clock for Fixed {
    fn now(&self) -> i64 {
        1_736_467_200
    }
}

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn package(text: &str) -> AurPackage {
    AurPackage {
        name: name(text),
        version: "1-1".to_owned(),
        description: String::new(),
        url: None,
        votes: 0,
        popularity: 0.0,
        out_of_date: None,
        maintainer: None,
        first_submitted: 0,
        last_modified: 0,
        depends: Vec::new(),
        make_depends: Vec::new(),
        opt_depends: Vec::new(),
        provides: Vec::new(),
        conflicts: Vec::new(),
        licenses: Vec::new(),
        keywords: Vec::new(),
    }
}

fn args(names: &[&str]) -> PackageInstall {
    PackageInstall {
        packages: names.iter().map(|text| name(text)).collect(),
        aur: true,
        confirm: Confirm::default(),
    }
}

fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::new()));
    snapshot
}

fn add(names: &[&str], aur: &dyn AurSource, changes: &mut ChangeSet) -> Result<(), Refused> {
    let args = args(names);
    let runtime = Runtime::new().unwrap();
    let catalogs = Catalogs::default();
    let sources = Sources {
        catalogs: &catalogs,
        runtime: &runtime,
        aur,
    };

    build(&Queueable::Install(&args), &sources, &system(), changes)
}

#[test]
fn queue_add_reads_the_aur_copy_once_for_every_name() {
    let copy = Copy::default();
    let mut changes = ChangeSet::default();

    add(&["yay-bin", "paru-bin"], &copy, &mut changes).unwrap();

    assert_eq!(copy.reads.load(Ordering::SeqCst), 1);
    assert_eq!(changes.queued::<AurPackages>().len(), 2);
}

#[test]
fn queue_add_refuses_a_name_the_aur_copy_lacks() {
    let copy = Copy::default();
    let mut changes = ChangeSet::default();

    let refused = add(&["yay-bin", "not-in-aur"], &copy, &mut changes).unwrap_err();

    assert!(matches!(refused, Refused::Unknown(_)), "{refused:?}");
    assert!(changes.is_empty(), "queued part of a refused change");
}

#[test]
fn queue_adds_within_a_day_download_at_most_once() {
    let folder = tempfile::tempdir().unwrap();
    let offline = Arc::new(Offline::default());
    let cache = Cache::new(
        folder.path().to_path_buf(),
        Arc::<Offline>::clone(&offline),
        Arc::new(Fixed),
    );

    for _ in 0..3_u8 {
        let mut changes = ChangeSet::default();
        let refused = add(&["yay-bin"], &cache, &mut changes).unwrap_err();
        assert!(matches!(refused, Refused::Unavailable(_)), "{refused:?}");
    }

    assert_eq!(offline.requests.load(Ordering::SeqCst), 1);
}
