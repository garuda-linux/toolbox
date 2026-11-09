use toolbox_core::domains::{DkmsBuilds, Packages};
use toolbox_core::features::kernels::{ModuleState, RunningKernel};
use toolbox_core::{ChangeSet, Queued, Snapshot, Toggle};
use toolbox_system::ProbeError;

use super::super::fixtures::{dkms, package, system, vboxhost_build};
use super::{Dkms, DkmsFailure, DkmsReading, Listing, Row, dkms_answer};

fn row<'rows, 'listing>(rows: &'rows [Row<'listing>], name: &str) -> &'rows Row<'listing> {
    rows.iter()
        .find(|row| row.kernel.name.as_str() == name)
        .unwrap()
}

#[test]
fn without_package_databases_there_is_no_listing() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.12.30").unwrap(),
        package: None,
    });

    assert!(Listing::of(&snapshot).is_none());
}

#[test]
fn a_row_has_the_kernel_its_state_and_its_modules() {
    let listing = Listing::of(&system()).unwrap();
    let changes = ChangeSet::default();
    let rows = listing.rows(&changes, &dkms());
    let linux = row(&rows, "linux");

    assert!(linux.running);
    assert!(linux.wanted);
    assert_eq!(linux.queued, None);
    assert!(!linux.lacks_headers);
    assert_eq!(linux.kernel.installed.as_deref(), Some("6.14.0-3"));

    let modules: Vec<(&str, ModuleState, bool)> = linux
        .modules
        .iter()
        .map(|module| {
            (
                module.status.module.as_str(),
                module.status.state,
                module.queued,
            )
        })
        .collect();

    assert_eq!(
        modules,
        [
            ("nvidia", ModuleState::Installed, false),
            ("vboxhost", ModuleState::Missing, false),
        ]
    );
    assert_eq!(linux.unbuilt(), 1);
    assert!(!linux.builds_queued());
    assert_eq!(listing.headers_missing(&changes), 1);
    assert_eq!(listing.modules_missing(&changes, &dkms()), 1);
}

#[test]
fn a_kernel_without_headers_has_no_modules() {
    let listing = Listing::of(&system()).unwrap();
    let rows = listing.rows(&ChangeSet::default(), &dkms());
    let lts = row(&rows, "linux-lts");

    assert!(lts.lacks_headers);
    assert!(lts.modules.is_empty());
    assert!(!row(&rows, "linux-zen").kernel.is_installed());
}

#[test]
fn modules_wait_for_dkms() {
    let listing = Listing::of(&system()).unwrap();
    let rows = listing.rows(&ChangeSet::default(), &DkmsReading::NotLoaded);

    assert!(row(&rows, "linux").modules.is_empty());
    assert_eq!(listing.all_builds(&DkmsReading::NotLoaded), []);
    assert_eq!(
        listing.modules_missing(&ChangeSet::default(), &DkmsReading::NotLoaded),
        0
    );
}

#[test]
fn queued_changes_show_on_the_rows_and_leave_the_counts() {
    let snapshot = system();
    let listing = Listing::of(&snapshot).unwrap();
    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("linux-zen"), Toggle::On, &snapshot);
    changes.want::<Packages>(package("linux-lts-headers"), Toggle::On, &snapshot);
    changes.want::<DkmsBuilds>(vboxhost_build(), Queued, &snapshot);

    let rows = listing.rows(&changes, &dkms());

    assert_eq!(row(&rows, "linux-zen").queued, Some(Toggle::On));
    assert!(row(&rows, "linux-zen").wanted, "a queued install wins");
    assert_eq!(row(&rows, "linux-lts").headers_queued, Some(Toggle::On));
    assert!(!row(&rows, "linux-lts").lacks_headers);
    assert!(row(&rows, "linux").builds_queued());
    assert_eq!(listing.headers_missing(&changes), 0);
    assert_eq!(listing.modules_missing(&changes, &dkms()), 0);
}

#[test]
fn a_queued_removal_makes_an_installed_kernel_unwanted() {
    let snapshot = system();
    let listing = Listing::of(&snapshot).unwrap();
    let mut changes = ChangeSet::default();

    changes.want::<Packages>(package("linux-lts"), Toggle::Off, &snapshot);

    let rows = listing.rows(&changes, &dkms());

    assert!(!row(&rows, "linux-lts").wanted);
    assert!(
        !row(&rows, "linux-zen").wanted,
        "nothing queued, not installed"
    );
}

#[test]
fn the_listing_finds_the_running_kernel_and_the_builds() {
    let snapshot = system();
    let listing = Listing::of(&snapshot).unwrap();
    let linux = listing.kernel(&package("linux")).unwrap();
    let lts = listing.kernel(&package("linux-lts")).unwrap();

    assert_eq!(
        listing.listed().running.map(|kernel| kernel.name.as_str()),
        Some("linux")
    );
    assert_eq!(super::release(&snapshot), "6.14.0-arch3-1");
    assert!(listing.kernel(&package("linux-nope")).is_none());
    assert_eq!(listing.builds(linux, &dkms()), [vboxhost_build()]);
    assert_eq!(listing.builds(lts, &dkms()), []);
}

#[test]
fn a_dkms_answer_becomes_its_state() {
    assert_eq!(
        dkms_answer(Err(ProbeError::Unavailable)),
        Ok(Dkms::NotInstalled)
    );
    assert_eq!(
        dkms_answer(Err(ProbeError::Failed("no output".to_owned()))),
        Err(DkmsFailure("no output".to_owned()))
    );
    assert_eq!(dkms_answer(Ok(Vec::new())), Ok(Dkms::Modules(Vec::new())));
}

#[test]
fn modules_stay_while_dkms_is_asked_again() {
    let listing = Listing::of(&system()).unwrap();
    let mut asked = dkms();
    asked.start_loading();

    assert_eq!(listing.all_builds(&asked), listing.all_builds(&dkms()));
    assert_eq!(listing.all_builds(&asked).len(), 1);
}
