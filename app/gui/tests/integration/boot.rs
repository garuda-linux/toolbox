//! Boot options with the menu, a chosen file and the Kernels proposal.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::domains::CurrentGrub;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::boot::{BootSetup, Chosen, FileChooser, FileKind};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState};
use toolbox_system::{BootMenuSource, Host, MenuRead};

use crate::common::{Fixture, TALL, queued_shown, window};
use crate::look::look_for;
use crate::system::{first_read, settle, shows, step_until};

/// `/etc/default/grub` of the Lima VM.
const DEFAULTS: &str = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;

/// The menu grub-mkconfig wrote on the Lima VM, with linux-lts and
/// linux-garuda.
const GRUB_CFG: &str = toolbox_plan::testing::grub::LIMA_GRUB_CFG;

struct Source;

struct Chooser;

impl BootMenuSource for Source {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        MenuRead::Known(GrubMenu::for_platform(GRUB_CFG, GrubPlatform::Efi))
    }
}

impl FileChooser for Chooser {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Chosen {
        Chosen::File("/usr/share/grub/themes/garuda/theme.txt".to_owned())
    }
}

/// The kernels of the Kernels tests, with GRUB.
fn system() -> Snapshot {
    let mut snapshot = crate::kernels::system(true);
    snapshot.insert(Host::default());
    snapshot.insert(CurrentGrub(GrubState::read(&defaults())));
    snapshot
}

/// The Lima VM's settings. Edited: without its `GRUB_THEME` line, as the
/// VM already uses the theme the chooser picks, so choosing it again
/// would change nothing.
fn defaults() -> String {
    DEFAULTS
        .lines()
        .filter(|line| !line.starts_with("GRUB_THEME="))
        .flat_map(|line| [line, "\n"])
        .collect()
}

pub fn app() -> Fixture {
    app_on(Theme::Dark)
}

/// The Boot Options page drawn on `theme`, with that theme's desktop look.
pub fn app_on(theme: Theme) -> Fixture {
    app_restoring_on(system(), Arc::new(Source), ChangeSet::default(), theme)
}

pub fn app_with(probed: Snapshot, menu: Arc<dyn BootMenuSource>) -> Fixture {
    app_restoring(probed, menu, ChangeSet::default())
}

/// With `changes` as the queue an earlier run left.
pub fn app_restoring(
    probed: Snapshot,
    menu: Arc<dyn BootMenuSource>,
    changes: ChangeSet,
) -> Fixture {
    app_restoring_on(probed, menu, changes, Theme::Dark)
}

/// As [`app_restoring`], drawn on `theme`.
pub fn app_restoring_on(
    probed: Snapshot,
    menu: Arc<dyn BootMenuSource>,
    changes: ChangeSet,
    theme: Theme,
) -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let look = look_for(&runtime, theme);
    let mut harness = window(TALL).with_theme(theme).build_eframe(move |cc| {
        let probe = move || (probed.clone(), Vec::new());
        App::new(
            cc,
            Setup {
                look,
                queue: Queue::restored_in_memory(changes),
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
    // GRUB's default of this capture boots linux-lts, which runs here and
    // would then be the default. The default here is an entry of linux-garuda,
    // which the system does not have, so linux-lts keeps its link.
    let garuda = "gnulinux-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6>gnulinux-linux-garuda-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6";
    let mut probed = system();
    probed.insert(CurrentGrub(GrubState::read(&defaults().replacen(
        "GRUB_DEFAULT=0",
        &format!("GRUB_DEFAULT='{garuda}'"),
        1,
    ))));
    let mut fixture = app_with(probed, Arc::new(Source));
    let harness = fixture.harness();

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num9);
    harness.run();
    harness
        .get_by_label("Set linux-lts as the default boot entry")
        .click();
    step_until(harness, "the offer", shows("Queue as Default"));
    settle(harness);
    harness.snapshot("boot/proposal_dark");
    harness.get_by_label("Queue as Default").click();
    harness.run();

    assert!(queued_shown(harness));
    assert!(harness.query_by_label("Queue as Default").is_none());
}

#[test]
fn a_chosen_theme_is_queued() {
    let mut fixture = app();
    let harness = fixture.harness();

    open_boot(harness);
    harness.get_by_label("Choose GRUB theme").scroll_to_me();
    harness.run();
    harness.get_by_label("Choose GRUB theme").click();
    step_until(harness, "the queued theme", queued_shown);
}
