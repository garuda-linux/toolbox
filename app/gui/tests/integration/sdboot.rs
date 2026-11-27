use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::{BootLoader, KernelCmdline, SdBootKernel};
use toolbox_core::features::boot::set_loader;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::App;
use toolbox_plan::sdboot::{LoaderEntry, LoaderSetting, LoaderSettings, SdBootMenu, sections};
use toolbox_system::{BootMenuSource, Host, MenuRead};

use crate::common::{Fixture, queued_shown};
use crate::system::step_until;

const LOADER_CONF: &str = toolbox_plan::testing::sdboot::LOADER_CONF;
const KERNEL_CMDLINE: &str = toolbox_state::testing::boot::KERNEL_CMDLINE;
const UKI: &[u8] = toolbox_plan::testing::sdboot::GARUDA_7_3_1_1_GARUDA_EFI_HEAD;
const FALLBACK: &[u8] = toolbox_plan::testing::sdboot::GARUDA_7_3_1_1_GARUDA_FALLBACK_EFI_HEAD;

struct Source;

impl BootMenuSource for Source {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        MenuRead::SdBoot(SdBootMenu {
            settings: LoaderSettings::parse(LOADER_CONF),
            entries: vec![
                entry("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK),
                entry("garuda-7.3.1-1-garuda.efi", UKI),
            ],
            ..SdBootMenu::default()
        })
    }
}

/// [`Source`], counting the reads.
#[derive(Default)]
struct Counted(AtomicUsize);

impl BootMenuSource for Counted {
    fn menu(&self, cancel: &Arc<AtomicBool>) -> MenuRead {
        self.0.fetch_add(1, Ordering::SeqCst);
        Source.menu(cancel)
    }
}

fn entry(file: &str, image: &[u8]) -> LoaderEntry {
    let osrel = sections(image)
        .into_iter()
        .find(|section| section.name == ".osrel")
        .unwrap();

    let start = usize::try_from(osrel.offset).unwrap();
    let end = start + usize::try_from(osrel.size).unwrap();
    let os_release = String::from_utf8(image[start..end].to_vec()).unwrap();

    LoaderEntry::uki(file, &os_release, None).unwrap()
}

fn system() -> Snapshot {
    let mut snapshot = crate::kernels::system(true);

    snapshot.insert(Host::default());
    snapshot.insert(BootLoader::SystemdBoot);
    snapshot.insert(SdBootKernel {
        cmdline: KernelCmdline::read(KERNEL_CMDLINE, false),
        theme: None,
    });
    snapshot
}

pub fn app() -> Fixture {
    crate::boot::app_with(system(), Arc::new(Source))
}

pub fn open_boot(harness: &mut Harness<'static, App>) {
    harness
        .get_by_role_and_label(Role::Tab, "Boot Options")
        .click();
    step_until(harness, "the systemd-boot menu", |harness| {
        harness
            .query_by_role_and_label(Role::ComboBox, "Default entry")
            .is_some()
    });
    harness.run();
}

#[test]
fn the_menu_is_read_once_the_page_opens() {
    let source = Arc::new(Counted::default());
    let mut fixture = crate::boot::app_with(system(), Arc::<Counted>::clone(&source));
    let harness = fixture.harness();

    open_boot(harness);

    assert!(harness.query_by_label("console=ttyS0,115200").is_some());
    assert_eq!(source.0.load(Ordering::SeqCst), 1, "the menu is read once");
}

/// No probe reads loader.conf. A restored loader change makes the window
/// read the menu with the Boot page closed, so a wish the file already
/// meets leaves the queue instead of running again.
#[test]
fn a_restored_loader_change_the_file_meets_leaves_without_the_page() {
    let mut changes = ChangeSet::default();
    set_loader(
        LoaderSetting::Timeout(None),
        &mut changes,
        &Snapshot::default(),
    );

    let source = Arc::new(Counted::default());
    let menu: Arc<dyn BootMenuSource> = Arc::<Counted>::clone(&source);
    let mut fixture = crate::boot::app_restoring(system(), menu, changes);
    let harness = fixture.harness();

    step_until(harness, "the met change to leave", |harness| {
        !queued_shown(harness)
    });

    assert_eq!(source.0.load(Ordering::SeqCst), 1);
}
