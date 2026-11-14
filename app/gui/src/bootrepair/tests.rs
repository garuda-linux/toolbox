use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Runtime;
use toolbox_core::features::bootrepair::page::{Booted, Facts, Offer, Progress};
use toolbox_core::features::bootrepair::{Refusal, RepairFacts};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_system::{DryRunRepair, ProbeError, RepairSource};

use super::{BootRepair, BootRepairSetup};
use crate::command::{AppCommand, BootRepairCommand};
use crate::page::PageId;
use crate::page_service::PageService as _;
use crate::testing::until;

#[derive(Default)]
struct Counted(AtomicUsize);

impl RepairSource for Counted {
    fn facts(&self) -> Result<RepairFacts, ProbeError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        DryRunRepair.facts()
    }
}

fn repair(forced: bool) -> (Runtime, Arc<Counted>, BootRepair) {
    let runtime = Runtime::new().unwrap();
    let source = Arc::new(Counted::default());
    let setup = BootRepairSetup {
        runtime: runtime.handle().clone(),
        source: Arc::<Counted>::clone(&source),
        forced,
    };

    let repair = BootRepair::new(Some(setup), egui::Context::default());
    (runtime, source, repair)
}

fn settle(repair: &mut BootRepair) {
    until("the disks were never read", || {
        repair.take();
        matches!(repair.facts(), Facts::Known(_))
    });
}

#[test]
fn an_installed_system_hides_the_page_and_reads_nothing() {
    let (_runtime, source, mut repair) = repair(false);

    repair.open();
    std::thread::sleep(Duration::from_millis(50));
    repair.take();

    assert!(!repair.offered());
    assert_eq!(*repair.facts(), Facts::NotAsked);
    assert_eq!(source.0.load(Ordering::SeqCst), 0);
}

#[test]
fn the_live_iso_offers_the_page_and_reads_once() {
    let (_runtime, source, mut repair) = repair(false);

    repair.see_live(Booted::Live);
    assert_eq!(repair.offer(), Offer::Live);

    repair.open();
    settle(&mut repair);
    repair.open();

    assert!(matches!(repair.facts(), Facts::Known(_)));
    assert_eq!(source.0.load(Ordering::SeqCst), 1);
}

#[test]
fn the_flag_shows_the_test_mode_and_a_run_reads_again_after_it_ended() {
    let (_runtime, source, mut repair) = repair(true);

    assert_eq!(repair.offer(), Offer::Test);
    repair.open();
    settle(&mut repair);

    repair.started();
    repair.plan_started();
    repair.plan_finished(false);

    assert_eq!(repair.progress(), Progress::Failed);
    until("the disks were never read again", || {
        source.0.load(Ordering::SeqCst) >= 2
    });

    assert_eq!(source.0.load(Ordering::SeqCst), 2);
    repair.forget();
    assert_eq!(repair.progress(), Progress::None);
}

#[test]
fn forget_is_the_services_and_a_refusal_goes_when_dismissed() {
    let mut repair = BootRepair::default();
    repair.set_refusal(Some(Refusal::NotOffered));

    let forgotten = repair.handle(&AppCommand::BootRepair(BootRepairCommand::Forget));
    assert!(forgotten.taken);
    assert!(repair.refusal().is_some());

    let dismissed = repair.receive(
        &AppCommand::DismissRefusal(PageId::BootRepair),
        &ChangeSet::default(),
        &Snapshot::default(),
    );

    assert!(dismissed.taken);
    assert!(repair.refusal().is_none());
}

/// The Console is App's.
#[test]
fn show_console_is_left_to_app() {
    let mut repair = BootRepair::default();

    let handled = repair.handle(&AppCommand::BootRepair(BootRepairCommand::ShowConsole));

    assert!(!handled.taken);
}
