use std::sync::atomic::{AtomicUsize, Ordering};

use toolbox_aur::{AurError, AurSource};
use toolbox_core::features::packages::aur::{AurIndex, AurPackage};
use toolbox_plan::PackageName;

use super::{names, remembered};

/// A copy of paru and yay-bin, or none with the error. Counts how often
/// it was asked through which door.
struct Copy {
    error: Option<AurError>,
    indexed: AtomicUsize,
    stored: AtomicUsize,
}

impl Copy {
    fn new(error: Option<AurError>) -> Self {
        Self {
            error,
            indexed: AtomicUsize::new(0),
            stored: AtomicUsize::new(0),
        }
    }

    fn answer(&self) -> Result<AurIndex, AurError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }

        Ok(AurIndex {
            packages: vec![package("paru"), package("yay-bin")],
            downloaded: 1_736_467_200,
            stale: false,
        })
    }
}

impl AurSource for Copy {
    fn index(&self) -> Result<AurIndex, AurError> {
        self.indexed.fetch_add(1, Ordering::SeqCst);
        self.answer()
    }

    fn stored(&self) -> Result<AurIndex, AurError> {
        self.stored.fetch_add(1, Ordering::SeqCst);
        self.answer()
    }
}

fn package(name: &str) -> AurPackage {
    AurPackage {
        name: PackageName::new(name).unwrap(),
        version: "1.0-1".to_owned(),
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

#[test]
fn the_names_come_from_the_stored_copy() {
    let copy = Copy::new(None);

    assert_eq!(names(&copy), ["paru", "yay-bin"]);
    assert_eq!(copy.stored.load(Ordering::SeqCst), 1);
    assert_eq!(copy.indexed.load(Ordering::SeqCst), 0);
}

#[test]
fn no_copy_gives_no_names() {
    for error in [
        AurError::Waiting { next: 1 },
        AurError::Download("offline".to_owned()),
        AurError::Storage("full".to_owned()),
    ] {
        assert_eq!(names(&Copy::new(Some(error))), Vec::<String>::new());
    }
}

#[test]
fn the_names_are_read_once_while_the_copy_stays() {
    let dir = tempfile::tempdir().unwrap();
    let reads = AtomicUsize::new(0);
    let read = || {
        reads.fetch_add(1, Ordering::SeqCst);
        vec!["paru".to_owned()]
    };

    assert_eq!(remembered(Some(dir.path()), || Some(7), read), ["paru"]);
    assert_eq!(remembered(Some(dir.path()), || Some(7), read), ["paru"]);
    assert_eq!(reads.load(Ordering::SeqCst), 1);

    remembered(Some(dir.path()), || Some(8), read);
    assert_eq!(reads.load(Ordering::SeqCst), 2);
}

#[test]
fn names_read_while_the_copy_changed_are_not_kept() {
    let dir = tempfile::tempdir().unwrap();
    let stamps = AtomicUsize::new(0);
    let stamp = || match stamps.fetch_add(1, Ordering::SeqCst) {
        0 => None,
        _ => Some(9),
    };

    remembered(Some(dir.path()), stamp, Vec::new);

    assert_eq!(
        super::super::cache::read_stamped(dir.path(), toolbox_cli::complete::Names::AurPackages, 9),
        None
    );
}
