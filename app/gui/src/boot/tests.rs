use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use tokio::runtime::Runtime;
use toolbox_core::Snapshot;
use toolbox_core::domains::CurrentGrub;
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState};
use toolbox_services::boot_menu::Menu;
use toolbox_system::{BootMenuSource, MenuRead};

use super::{Boot, BootSetup, Chosen, FileChooser, FileKind, Picked};
use crate::page::PageId;
use crate::page_service::PageService;
use crate::testing::captures::GRUB_CFG;
use crate::testing::until;

/// Counts its reads and answers the menu of [`GRUB_CFG`].
#[derive(Default)]
struct Source(AtomicUsize);

/// Picks the same file every time.
struct Chooser;

/// Counts its dialogs. Each stays open until `open` is cleared, then the
/// user cancels it.
#[derive(Default)]
struct Held {
    asked: AtomicUsize,
    open: AtomicBool,
}

/// Gives the same answer every time, as a desktop without a portal does.
struct Broken(Chosen);

impl BootMenuSource for Source {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        self.0.fetch_add(1, Ordering::SeqCst);
        MenuRead::Known(GrubMenu::for_platform(GRUB_CFG, GrubPlatform::Efi))
    }
}

impl FileChooser for Chooser {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Chosen {
        Chosen::File("/usr/share/grub/themes/garuda/theme.txt".to_owned())
    }
}

impl FileChooser for Held {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Chosen {
        self.asked.fetch_add(1, Ordering::SeqCst);
        until("the dialog was never closed", || {
            !self.open.load(Ordering::SeqCst)
        });
        Chosen::Cancelled
    }
}

impl FileChooser for Broken {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Chosen {
        self.0.clone()
    }
}

fn boot(runtime: &Runtime, source: &Arc<Source>) -> Boot {
    Boot::new(
        Some(BootSetup {
            runtime: runtime.handle().clone(),
            menu: Arc::<Source>::clone(source),
            chooser: Arc::new(Chooser),
        }),
        egui::Context::default(),
    )
}

/// Takes results until `done` holds, and returns the last picked file.
fn settle(boot: &mut Boot, done: impl Fn(&Boot, Option<&Picked>) -> bool) -> Option<Picked> {
    let mut picked = None;
    until("the job never ended", || {
        if let Some(file) = boot.take() {
            picked = Some(file);
        }

        done(boot, picked.as_ref())
    });
    picked
}

#[test]
fn the_menu_is_read_only_once_the_page_opens_on_a_grub_system() {
    let runtime = Runtime::new().unwrap();
    let source = Arc::new(Source::default());
    let mut boot = boot(&runtime, &source);

    boot.open(false);
    assert_eq!(*boot.menu(), Menu::NotAsked);

    boot.open(true);
    assert_eq!(*boot.menu(), Menu::Loading);
    settle(&mut boot, |boot, _| boot.menu().known().is_some());

    boot.open(true);
    assert_eq!(
        source.0.load(Ordering::SeqCst),
        1,
        "opened again, read once"
    );
}

#[test]
fn a_finished_plan_reads_an_asked_menu_again() {
    let runtime = Runtime::new().unwrap();
    let source = Arc::new(Source::default());
    let mut boot = boot(&runtime, &source);

    boot.plan_finished();
    assert_eq!(*boot.menu(), Menu::NotAsked, "never asked, never read");

    boot.open(true);
    settle(&mut boot, |boot, _| boot.menu().known().is_some());
    boot.plan_finished();
    settle(&mut boot, |_, _| source.0.load(Ordering::SeqCst) == 2);

    assert!(
        boot.menu().known().is_some(),
        "the old menu shows meanwhile"
    );
}

#[test]
fn a_chosen_file_comes_back_with_its_kind() {
    let runtime = Runtime::new().unwrap();
    let source = Arc::new(Source::default());
    let mut boot = boot(&runtime, &source);

    boot.choose(FileKind::Theme, "Choose a GRUB Theme", "GRUB theme");
    let picked = settle(&mut boot, |_, picked| picked.is_some()).unwrap();

    assert_eq!(picked.kind, FileKind::Theme);
    assert_eq!(picked.path, "/usr/share/grub/themes/garuda/theme.txt");
}

/// A second dialog would drop the first one's answer.
#[test]
fn a_second_choose_waits_until_the_open_dialog_is_answered() {
    let runtime = Runtime::new().unwrap();
    let chooser = Arc::new(Held::default());
    chooser.open.store(true, Ordering::SeqCst);
    let mut boot = Boot::new(
        Some(BootSetup {
            runtime: runtime.handle().clone(),
            menu: Arc::new(Source::default()),
            chooser: Arc::<Held>::clone(&chooser),
        }),
        egui::Context::default(),
    );

    boot.choose(FileKind::Theme, "Choose a GRUB Theme", "GRUB theme");
    until("the dialog never opened", || {
        chooser.asked.load(Ordering::SeqCst) == 1
    });
    boot.choose(FileKind::Background, "Choose a picture", "Picture");
    chooser.open.store(false, Ordering::SeqCst);
    settle(&mut boot, |boot, _| !boot.choosing());

    boot.choose(FileKind::Background, "Choose a picture", "Picture");
    until("the cancelled dialog blocked the next one", || {
        chooser.asked.load(Ordering::SeqCst) == 2
    });
    settle(&mut boot, |boot, _| !boot.choosing());

    assert_eq!(chooser.asked.load(Ordering::SeqCst), 2);
}

fn answered(chosen: Chosen) -> bool {
    let runtime = Runtime::new().unwrap();
    let mut boot = Boot::new(
        Some(BootSetup {
            runtime: runtime.handle().clone(),
            menu: Arc::new(Source::default()),
            chooser: Arc::new(Broken(chosen)),
        }),
        egui::Context::default(),
    );

    boot.choose(FileKind::Theme, "Choose a GRUB Theme", "GRUB theme");
    settle(&mut boot, |boot, _| !boot.choosing());
    boot.chooser_failed()
}

/// A dialog that never opened says so, a cancelled one is no failure.
#[test]
fn a_dialog_that_could_not_open_is_told_from_a_cancelled_one() {
    assert_eq!(
        [answered(Chosen::Failed), answered(Chosen::Cancelled)],
        [true, false]
    );
}

#[test]
fn opening_the_kernels_page_reads_the_menu_its_default_comes_from() {
    let runtime = Runtime::new().unwrap();
    let source = Arc::new(Source::default());
    let mut boot = boot(&runtime, &source);
    let mut snapshot = Snapshot::default();
    snapshot.insert(CurrentGrub(GrubState::read("")));

    PageService::take(&mut boot, PageId::Kernels, &snapshot);
    settle(&mut boot, |boot, _| boot.menu().known().is_some());

    assert_eq!(source.0.load(Ordering::SeqCst), 1);
}
