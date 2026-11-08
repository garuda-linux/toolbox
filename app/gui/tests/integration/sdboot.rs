use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::{BootLoader, KernelCmdline, SdBootKernel};
use toolbox_gui::App;
use toolbox_plan::sdboot::{LoaderEntry, LoaderSettings, SdBootMenu, sections};
use toolbox_system::{BootMenuSource, Host, MenuRead};

use crate::common::Fixture;
use crate::system::step_until;

const LOADER_CONF: &str = include_str!("../../../../crates/plan/src/sdboot/fixtures/loader.conf");
const KERNEL_CMDLINE: &str =
    include_str!("../../../../crates/state/src/boot/fixtures/kernel-cmdline");
const UKI: &[u8] =
    include_bytes!("../../../../crates/plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head");
const FALLBACK: &[u8] = include_bytes!(
    "../../../../crates/plan/src/sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head"
);

struct Source;

impl BootMenuSource for Source {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        MenuRead::SdBoot(SdBootMenu {
            settings: LoaderSettings::parse(LOADER_CONF),
            entries: vec![
                entry("garuda-7.3.1-1-garuda-fallback.efi", FALLBACK),
                entry("garuda-7.3.1-1-garuda.efi", UKI),
            ],
        })
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
    let mut fixture = app();
    let harness = fixture.harness();

    open_boot(harness);

    assert!(harness.query_by_label("console=ttyS0,115200").is_some());
}
