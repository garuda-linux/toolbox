//! The Kernels page with a running default kernel: its row shows Running
//! and Default side by side, in English, German and at the smallest window.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::{Key, Modifiers, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_core::ChangeSet;
use toolbox_core::domains::CurrentGrub;
use toolbox_core::features::kernels::{RunningKernel, parse_status};
use toolbox_gui::boot::{BootSetup, Chosen, FileChooser, FileKind};
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup, reader, snapshot};
use toolbox_plan::{GrubMenu, GrubPlatform, GrubState, KernelRelease};
use toolbox_services::dkms::Dkms;
use toolbox_system::{BootMenuSource, MenuRead};

use crate::common::{SMALLEST, TALL, window};
use crate::kernels::{DKMS_STATUS, package, system};
use crate::look::look_for;
use crate::system::{first_read, shows, step_until};

/// Reads the menu of the Lima capture, as the Boot page would.
struct LimaMenu;

impl BootMenuSource for LimaMenu {
    fn menu(&self, _cancel: &Arc<AtomicBool>) -> MenuRead {
        MenuRead::Known(GrubMenu::for_platform(
            toolbox_plan::testing::grub::LIMA_GRUB_CFG,
            GrubPlatform::Efi,
        ))
    }
}

/// Never asked, the menu of this test asks for no file.
struct NoFile;

impl FileChooser for NoFile {
    fn choose(&self, _kind: FileKind, _title: &str, _filter: &str) -> Chosen {
        Chosen::Cancelled
    }
}

/// The Lima GRUB menu's default set to the advanced entry of linux-lts.
fn lts_default() -> String {
    let grub = toolbox_plan::testing::grub::LIMA_DEFAULT_GRUB;
    assert!(
        grub.contains("GRUB_DEFAULT=0"),
        "the Lima GRUB file has no GRUB_DEFAULT=0 to replace"
    );
    grub.replacen(
        "GRUB_DEFAULT=0",
        "GRUB_DEFAULT='gnulinux-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6>gnulinux-linux-lts-advanced-cd2fff21-4bb7-40f3-a158-406f545e11e6'",
        1,
    )
}

/// linux-lts runs and is the default of the GRUB menu, in `language`.
fn running_default(
    theme: Theme,
    language: &'static str,
    labels: [&'static str; 3],
) -> (Harness<'static, App>, Runtime) {
    let mut system = system(false);
    system.insert(RunningKernel {
        release: KernelRelease::new("6.12.9-1-lts").unwrap(),
        package: Some(package("linux-lts")),
    });
    system.insert(CurrentGrub(GrubState::read(&lts_default())));

    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let look = look_for(&runtime, theme);
    let menu: Arc<dyn BootMenuSource> = Arc::new(LimaMenu);
    let mut harness = window(TALL).with_theme(theme).build_eframe(move |cc| {
        let probe = move || (system.clone(), Vec::new());
        let dkms = || Dkms::Modules(parse_status(DKMS_STATUS));

        App::new(
            cc,
            Setup {
                look,
                queue: Queue::in_memory(ChangeSet::default()),
                system_language: language.parse().unwrap(),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                dkms: Some(reader::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    "dkms",
                    dkms,
                )),
                boot: Some(BootSetup {
                    runtime: handle.clone(),
                    menu: Arc::clone(&menu),
                    chooser: Arc::new(NoFile),
                }),
                ..Setup::default()
            },
        )
    });

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num9);
    first_read(&mut harness);
    step_until(&mut harness, "the dkms answer", shows(labels[2]));
    step_until(&mut harness, "the default", shows(labels[1]));
    harness.run();
    (harness, runtime)
}

#[test]
fn a_running_default_kernel_shows_running_then_default() {
    let (mut harness, _runtime) = running_default(
        Theme::Dark,
        "en-US",
        ["Running", "Default", "nvidia 550.90.07: installed"],
    );

    let running = harness.get_by_label("Running").rect();
    let default = harness.get_by_label("Default").rect();

    assert!(
        running.right() <= default.left(),
        "Running ends at {} and Default starts at {}",
        running.right(),
        default.left(),
    );
    harness.snapshot("kernels/running_default_dark");
}

#[test]
fn a_running_default_kernel_keeps_both_states_in_german() {
    let (mut harness, _runtime) = running_default(
        Theme::Dark,
        "de-DE",
        ["Läuft", "Standard", "nvidia 550.90.07: installiert"],
    );

    assert!(harness.query_by_label("Läuft").is_some());
    harness.snapshot("kernels/running_default_german");
}

#[test]
fn a_running_default_kernel_keeps_both_states_at_the_smallest_window() {
    let (mut harness, _runtime) = running_default(
        Theme::Dark,
        "en-US",
        ["Running", "Default", "nvidia 550.90.07: installed"],
    );

    harness.set_size(egui::vec2(SMALLEST.0, SMALLEST.1));
    harness.run();
    harness.get_by_label("Running").scroll_to_me();
    harness.run();

    harness.snapshot("kernels/running_default_smallest");
}

#[test]
fn a_running_default_kernel_keeps_both_states_in_german_at_the_smallest_window() {
    let (mut harness, _runtime) = running_default(
        Theme::Dark,
        "de-DE",
        ["Läuft", "Standard", "nvidia 550.90.07: installiert"],
    );

    harness.set_size(egui::vec2(SMALLEST.0, SMALLEST.1));
    harness.run();
    harness.get_by_label("Läuft").scroll_to_me();
    harness.run();

    harness.snapshot("kernels/running_default_german_smallest");
}
