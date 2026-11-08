use std::collections::BTreeMap;
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use tokio::runtime::Runtime;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::kernels::{
    KernelReleases, LinuxPackages, LocalPackage, RepoPackage, RunningKernel, parse_status,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::command::{Command, KernelCommand, KernelEdit};
use toolbox_gui::pages::kernels::KernelsPage;
use toolbox_gui::pages::kernels::view::{Dkms, DkmsReading};
use toolbox_gui::queue::Queue;
use toolbox_gui::testing::{PageFixture, PageState};
use toolbox_gui::{App, Setup, reader, snapshot};
use toolbox_plan::{KernelRelease, PackageName};

use crate::common::{TALL, checkbox as switch, packages, window};
use crate::system::{first_read, shows, step_until};

/// What dkms lists for [`system`]: `nvidia` built for `linux`.
const DKMS_STATUS: &str =
    "nvidia/550.90.07, 6.14.0-arch1-1, x86_64: installed\nvboxhost/7.1.6_OSE: added\n";

fn package(name: &str) -> PackageName {
    name.parse().unwrap()
}

fn repo(repo: &str, name: &str, version: &str) -> RepoPackage {
    RepoPackage {
        repo: repo.to_owned(),
        name: package(name),
        version: version.to_owned(),
        description: format!("About {name}"),
    }
}

fn local(name: &str, version: &str) -> LocalPackage {
    LocalPackage {
        name: package(name),
        version: version.to_owned(),
        description: String::new(),
    }
}

/// `linux` runs with its headers, `linux-lts` lacks them, `linux-zen` is
/// offered. `running` says which kernel booted, if anyone knows.
pub fn system(running: bool) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-headers", "6.14.0-4"),
            repo("core", "linux-lts", "6.12.9-1"),
            repo("core", "linux-lts-headers", "6.12.9-1"),
            repo("extra", "linux-zen", "6.13.8.zen1-1"),
            repo("extra", "linux-zen-headers", "6.13.8.zen1-1"),
        ],
        local: vec![
            local("linux", "6.14.0-4"),
            local("linux-headers", "6.14.0-4"),
            local("linux-lts", "6.12.9-1"),
        ],
    });
    snapshot.insert(InstalledPackages(packages(&[
        "linux",
        "linux-headers",
        "linux-lts",
    ])));
    snapshot.insert(KernelReleases(BTreeMap::from([
        (
            package("linux"),
            KernelRelease::new("6.14.0-arch1-1").unwrap(),
        ),
        (
            package("linux-lts"),
            KernelRelease::new("6.12.9-1-lts").unwrap(),
        ),
    ])));

    if running {
        snapshot.insert(RunningKernel {
            release: "6.14.0-arch1-1".to_owned(),
            package: Some(package("linux")),
        });
    }

    snapshot
}

pub fn kernels(system: Snapshot) -> (Harness<'static, App>, Runtime) {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();

    let mut harness = window(TALL).build_eframe(move |cc| {
        let probe = move || (system.clone(), Vec::new());
        let dkms = || Dkms::Modules(parse_status(DKMS_STATUS));

        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
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
                ..Setup::default()
            },
        )
    });

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num9);
    first_read(&mut harness);
    step_until(
        &mut harness,
        "the dkms answer",
        shows("nvidia 550.90.07: installed"),
    );
    harness.run();
    (harness, runtime)
}

fn page(system: Snapshot) -> Harness<'static, PageState<KernelsPage>> {
    let fixture = PageFixture {
        dkms: DkmsReading::Loaded(Arc::new(Dkms::Modules(parse_status(DKMS_STATUS)))),
        ..PageFixture::with_snapshot(system)
    };

    let mut harness = fixture.harness(KernelsPage);
    harness.run();
    harness
}

#[test]
fn installed_and_offered_kernels_show_with_the_running_one() {
    let harness = page(system(true));

    assert!(
        harness
            .query_by_role_and_label(Role::List, "Installed")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::List, "Available")
            .is_some()
    );
    assert!(harness.query_by_label("Running").is_some());
    assert!(
        harness
            .query_by_label("Running kernel: 6.14.0-arch1-1")
            .is_some()
    );
    assert!(switch(&harness, "linux").accesskit_node().is_disabled());
    assert!(!switch(&harness, "linux-lts").accesskit_node().is_disabled());
}

#[test]
fn a_click_queues_a_kernel_with_its_headers_and_takes_it_back() {
    let (mut harness, _runtime) = kernels(system(true));

    // The row sits under the window's foot until scrolled into sight.
    switch(&harness, "linux-zen").scroll_to_me();
    harness.run();
    switch(&harness, "linux-zen").click();
    harness.run();
    assert!(harness.query_by_label("Installs with Apply").is_some());
    assert!(harness.query_by_label("changes queued").is_some());

    // The Changes band covers the page's foot, where the row lies.
    switch(&harness, "linux-zen").scroll_to_me();
    harness.run();
    switch(&harness, "linux-zen").click();
    harness.run();
    assert!(harness.query_by_label("Installs with Apply").is_none());
}

#[test]
fn a_removal_without_a_known_running_kernel_is_refused_until_dismissed() {
    let (mut harness, _runtime) = kernels(system(false));

    switch(&harness, "linux-lts").click();
    harness.run();
    assert!(
        harness
            .query_by_label_contains("The kernel that runs right now is not known")
            .is_some()
    );
    assert!(harness.query_by_label("Removes with Apply").is_none());

    harness.get_by_label("Dismiss").click();
    harness.run();
    assert!(
        harness
            .query_by_label_contains("The kernel that runs right now is not known")
            .is_none()
    );
}

#[test]
fn missing_headers_are_offered_and_queue() {
    let (mut harness, _runtime) = kernels(system(true));
    assert!(harness.query_by_label("Headers missing").is_some());

    harness.get_by_label("Queue all headers").click();
    harness.run();

    assert!(
        harness
            .query_by_label("Headers install with Apply")
            .is_some()
    );
    assert!(harness.query_by_label("Queue all headers").is_none());

    harness.get_by_label("Don't install headers").click();
    harness.run();
    assert!(harness.query_by_label("Headers missing").is_some());
}

#[test]
fn a_missing_module_build_queues_from_its_kernel() {
    let (mut harness, _runtime) = kernels(system(true));
    assert!(
        harness
            .query_by_label("vboxhost 7.1.6_OSE: missing")
            .is_some()
    );

    harness.get_by_label("Queue module builds").click();
    harness.run();

    assert!(
        harness
            .query_by_label("vboxhost 7.1.6_OSE: build queued")
            .is_some()
    );
    assert!(harness.query_by_label("Queue all builds").is_none());
}

#[test]
fn without_package_databases_the_page_says_so() {
    let harness = page(Snapshot::default());

    assert!(
        harness
            .query_by_label_contains("Kernels are not available on this system")
            .is_some()
    );
}

#[test]
fn a_click_on_an_offered_kernel_asks_to_install_it() {
    let mut harness = page(system(true));

    // The installed kernels fill the window, so the offered one lies below.
    switch(&harness, "linux-zen").scroll_to_me();
    harness.run();
    switch(&harness, "linux-zen").click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(KernelCommand::Edit(KernelEdit::Toggle {
            kernel: package("linux-zen"),
            on: true,
        }))]
    );
}

#[test]
fn the_version_and_the_running_note_are_secondary_text() {
    let harness = page(system(true));
    let secondary = toolbox_gui_kit::tokens::current(&harness.ctx).secondary;
    let colour = |text| toolbox_gui_kit::testing::text_colour(&harness, text);

    assert_eq!(colour("6.14.0-4 from core"), Some(secondary));
    assert_eq!(
        colour(
            "This kernel runs right now, so it stays. Restart with another kernel to remove it."
        ),
        Some(secondary)
    );
}

#[test]
fn a_long_module_line_wraps_and_keeps_the_switches_in_the_card() {
    let (harness, _runtime) = kernels(system(true));
    let builds = harness.get_by_label("Queue module builds").rect();
    let lts = harness
        .get_by_role_and_label(Role::CheckBox, "linux-lts")
        .rect();

    assert!(
        builds.right() < lts.left(),
        "the builds button ends at {} past the right column at {}",
        builds.right(),
        lts.left(),
    );
    assert!(
        lts.right() <= harness.ctx.content_rect().right() - 40.0,
        "the switches end at {}, outside the card",
        lts.right(),
    );
}
