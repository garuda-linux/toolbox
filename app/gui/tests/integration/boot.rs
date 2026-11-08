//! Boot options with its real state: the menu read once the page opens,
//! a file from the chooser, the Kernels page's proposal and the
//! snapshots.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::domains::CurrentGrub;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::boot::{BootSetup, FileChooser, FileKind};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::{GrubMenu, GrubState};
use toolbox_system::{BootMenuSource, Host, MenuRead};

use crate::common::{Fixture, TALL, window};
use crate::system::{first_read, shows, step_until};

const DEFAULTS: &str = "\
GRUB_DEFAULT=gnulinux-simple-1
GRUB_TIMEOUT=5
GRUB_CMDLINE_LINUX_DEFAULT=\"quiet loglevel=3\"
";

const GRUB_CFG: &str = "\
menuentry 'Garuda Linux' $menuentry_id_option 'gnulinux-simple-1' {
}
submenu 'Advanced options' $menuentry_id_option 'gnulinux-advanced-1' {
	menuentry 'Garuda Linux, with Linux linux-lts' $menuentry_id_option 'gnulinux-linux-lts-advanced-1' {
	}
}
";

struct Source;

struct Chooser;

impl BootMenuSource for Source {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        MenuRead::Known(GrubMenu::from_config(GRUB_CFG))
    }
}

impl FileChooser for Chooser {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Option<String> {
        Some("/usr/share/grub/themes/garuda/theme.txt".to_owned())
    }
}

/// The kernels of the Kernels tests, with GRUB.
fn system() -> Snapshot {
    let mut snapshot = crate::kernels::system(true);
    snapshot.insert(Host::default());
    snapshot.insert(CurrentGrub(GrubState::read(DEFAULTS)));
    snapshot
}

pub fn app() -> Fixture {
    app_with(system(), Arc::new(Source))
}

pub fn app_with(probed: Snapshot, menu: Arc<dyn BootMenuSource>) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let mut harness = window(TALL).build_eframe(move |cc| {
        let probe = move || (probed.clone(), Vec::new());
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                boot: Some(BootSetup {
                    runtime: handle.clone(),
                    menu: Arc::clone(&menu),
                    chooser: Arc::new(Chooser),
                }),
                ..Setup::default()
            },
        )
    });

    first_read(&mut harness);

    Fixture::new(harness, runtime)
}

pub fn open_boot(harness: &mut Harness<'static, App>) {
    harness
        .get_by_role_and_label(Role::Tab, "Boot Options")
        .click();
    step_until(harness, "the menu", |harness| {
        harness
            .query_by_label_contains("The menu entry that is booted")
            .is_some()
    });
    harness.run();
}

#[test]
fn the_kernels_page_proposes_a_kernel_as_the_default() {
    let mut fixture = app();
    let harness = fixture.harness();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num9);
    harness.run();
    harness
        .get_by_label("Set linux-lts as the default boot entry")
        .click();
    step_until(harness, "the offer", shows("Queue as default"));
    harness.get_by_label("Queue as default").click();
    harness.run();

    assert!(harness.query_by_label("change queued").is_some());
    assert!(harness.query_by_label("Queue as default").is_none());
}

#[test]
fn a_chosen_theme_is_queued() {
    let mut fixture = app();
    let harness = fixture.harness();

    open_boot(harness);
    harness.get_by_label("Choose GRUB theme").scroll_to_me();
    harness.run();
    harness.get_by_label("Choose GRUB theme").click();
    step_until(harness, "the queued theme", shows("change queued"));
}
