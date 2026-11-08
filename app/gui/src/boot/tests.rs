use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use tokio::runtime::Runtime;
use toolbox_plan::GrubMenu;
use toolbox_system::{BootMenuSource, Menu, MenuRead};

use super::{Boot, BootSetup, FileChooser, FileKind, Picked};
use crate::testing::until;

const GRUB_CFG: &str = "menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {\n}\n";

/// Counts its reads and answers a one-entry menu.
#[derive(Default)]
struct Source(AtomicUsize);

/// Picks the same file every time.
struct Chooser;

impl BootMenuSource for Source {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        self.0.fetch_add(1, Ordering::SeqCst);
        MenuRead::Known(GrubMenu::from_config(GRUB_CFG))
    }
}

impl FileChooser for Chooser {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Option<String> {
        Some("/usr/share/grub/themes/garuda/theme.txt".to_owned())
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

    boot.choose(FileKind::Theme, "Choose a GRUB theme", "GRUB theme");
    let picked = settle(&mut boot, |_, picked| picked.is_some()).unwrap();

    assert_eq!(picked.kind, FileKind::Theme);
    assert_eq!(picked.path, "/usr/share/grub/themes/garuda/theme.txt");
}
