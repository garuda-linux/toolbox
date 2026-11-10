use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Runtime;
use toolbox_aur::{AurError, AurSource};
use toolbox_core::features::packages::aur::AurIndex;
use toolbox_core::features::packages::history::HistoryAction;
use toolbox_core::features::packages::{
    IndexEntry, InstallReason, Installed, Origin, PackageDetails, PackageIndex,
};
use toolbox_plan::PackageName;
use toolbox_system::{PackageIndexSource, ProbeError, RecentLogSource};

use super::{Packages, PackagesSetup, Read};
use crate::testing::until;

const LIMA_LOG: &str = toolbox_state::testing::status::pacman_log::PACMAN_LOG;

/// gimp installed as in the Lima VM, whose pacman log names it once.
#[derive(Default)]
struct Machine {
    index_reads: AtomicUsize,
}

struct Log;

/// The AUR's copy as a fake answers it, counting the reads.
#[derive(Default)]
struct Aur {
    reads: AtomicUsize,
    answer: Option<AurError>,
}

impl AurSource for Aur {
    fn index(&self) -> Result<AurIndex, AurError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.answer.clone().map_or_else(
            || {
                Ok(AurIndex {
                    packages: Vec::new(),
                    downloaded: 1_700_000_000,
                    stale: false,
                })
            },
            Err,
        )
    }
}

impl PackageIndexSource for Machine {
    fn index(&self) -> Result<PackageIndex, ProbeError> {
        self.index_reads.fetch_add(1, Ordering::SeqCst);
        Ok(PackageIndex {
            entries: vec![IndexEntry {
                name: name("gimp"),
                description: "GNU Image Manipulation Program".to_owned(),
                version: "3.2.6-2".to_owned(),
                origin: Origin::Repository("extra".to_owned()),
                size: 1,
                installed: Some(Installed {
                    version: "3.2.6-2".to_owned(),
                    reason: InstallReason::Explicit,
                }),
            }],
            local: Vec::new(),
        })
    }

    fn details(&self, package: &PackageName) -> Result<Option<PackageDetails>, ProbeError> {
        if package.as_str() != "gimp" {
            return Ok(None);
        }

        Ok(Some(PackageDetails {
            name: package.clone(),
            description: String::new(),
            version: "3.2.6-2".to_owned(),
            origin: Origin::Repository("extra".to_owned()),
            size: 1,
            url: None,
            licenses: Vec::new(),
            groups: Vec::new(),
            depends: Vec::new(),
            optional: Vec::new(),
            required_by: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            installed: None,
        }))
    }
}

impl RecentLogSource for Log {
    fn recent(&self) -> Result<String, ProbeError> {
        Ok(LIMA_LOG.to_owned())
    }
}

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn packages(runtime: &Runtime, machine: &Arc<Machine>) -> Packages {
    with_aur(runtime, machine, &Arc::new(Aur::default()))
}

fn with_aur(runtime: &Runtime, machine: &Arc<Machine>, aur: &Arc<Aur>) -> Packages {
    Packages::new(
        Some(PackagesSetup {
            runtime: runtime.handle().clone(),
            source: Arc::<Machine>::clone(machine),
            log: Arc::new(Log),
            aur: Arc::<Aur>::clone(aur),
        }),
        egui::Context::default(),
    )
}

fn settle(packages: &mut Packages, done: impl Fn(&Packages) -> bool) {
    until("the read never ended", || {
        packages.take();
        done(packages)
    });
}

#[test]
fn nothing_is_read_before_the_page_opens() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut packages = packages(&runtime, &machine);

    packages.plan_finished();
    std::thread::sleep(Duration::from_millis(50));
    packages.take();

    assert_eq!(machine.index_reads.load(Ordering::SeqCst), 0);
    assert!(matches!(packages.index(), Read::Loading));
}

#[test]
fn opening_reads_the_index_once_and_a_plan_reads_it_again() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut packages = packages(&runtime, &machine);

    packages.open();
    packages.open();
    settle(&mut packages, |shown| shown.index().known().is_some());
    assert_eq!(
        packages
            .index()
            .known()
            .map(|catalog| catalog.entries().len()),
        Some(1)
    );

    packages.plan_finished();
    until("the second read never ran", || {
        machine.index_reads.load(Ordering::SeqCst) == 2
    });
}

#[test]
fn showing_a_package_reads_its_details_and_its_history() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut packages = packages(&runtime, &machine);

    packages.show(&name("gimp"));
    settle(&mut packages, |shown| {
        shown.details().is_some_and(|details| {
            details.info != Read::Loading && details.history != Read::Loading
        })
    });

    let details = packages.details().unwrap();
    assert!(matches!(&details.info, Read::Known(Some(info)) if info.version == "3.2.6-2"));
    assert!(matches!(
        &details.history,
        Read::Known(lines) if lines.len() == 1 && lines[0].action == HistoryAction::Installed
    ));
}

#[test]
fn an_answer_for_a_package_the_view_left_is_dropped() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut packages = packages(&runtime, &machine);

    packages.show(&name("gimp"));
    packages.show(&name("gone"));
    settle(&mut packages, |shown| {
        shown
            .details()
            .is_some_and(|details| details.info != Read::Loading)
    });

    let details = packages.details().unwrap();
    assert_eq!(details.package, name("gone"));
    assert_eq!(details.info, Read::Known(None));
}

#[test]
fn the_aur_is_read_only_once_its_chip_asks_and_then_once() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let aur = Arc::new(Aur::default());
    let mut packages = with_aur(&runtime, &machine, &aur);

    packages.open();
    packages.plan_finished();
    settle(&mut packages, |shown| shown.index().known().is_some());
    assert_eq!(aur.reads.load(Ordering::SeqCst), 0);
    assert!(matches!(packages.aur(), Read::Loading));

    packages.open_aur();
    packages.open_aur();
    settle(&mut packages, |shown| shown.aur().known().is_some());
    assert_eq!(
        packages.aur().known().map(|catalog| catalog.downloaded()),
        Some(1_700_000_000)
    );

    packages.plan_finished();
    std::thread::sleep(Duration::from_millis(50));
    packages.take();
    assert_eq!(aur.reads.load(Ordering::SeqCst), 1);
}

#[test]
fn an_aur_copy_that_waits_for_its_day_is_unavailable_and_a_failed_download_failed() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());

    for (answer, wanted) in [
        (AurError::Waiting { next: 1 }, Read::Unavailable),
        (AurError::Download("no network".to_owned()), Read::Failed),
    ] {
        let aur = Arc::new(Aur {
            reads: AtomicUsize::new(0),
            answer: Some(answer),
        });

        let mut packages = with_aur(&runtime, &machine, &aur);
        packages.open_aur();
        settle(&mut packages, |shown| !matches!(shown.aur(), Read::Loading));

        assert_eq!(
            std::mem::discriminant(packages.aur()),
            std::mem::discriminant(&wanted)
        );
    }
}

#[test]
fn the_aur_names_land_once_with_the_copy() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut packages = packages(&runtime, &machine);

    assert!(!packages.aur_landed());
    packages.open_aur();
    settle(&mut packages, |shown| shown.aur().known().is_some());

    assert!(packages.aur_names().is_some());
    assert!(packages.aur_landed());
    assert!(!packages.aur_landed(), "the landing counts once");
}
