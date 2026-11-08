//! The Config Files page on the self-test's own catalog and files: the
//! list by section, a folder's files, the view of a guarded file, and
//! the snapshots.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_catalog::{ConfigFileCatalog, LoadProblem};
use toolbox_core::ChangeSet;
use toolbox_gui::configs::ConfigsSetup;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup};
use toolbox_plan::ConfigFilePath;
use toolbox_system::{DryRunEditor, EditFolder, HelperFetch, SystemConfigFiles};

use crate::common::{TALL, window};
use crate::system::{settle, step_until};

/// The fixture the self-test uses too.
const FIXTURE: &str = "../../tests/selftest/configs";

/// Every file of the fixture is readable, so the helper is never asked.
struct NoHelper;

/// The app on Config Files. The runtime and the folder live as long as
/// the harness.
pub struct Fixture {
    harness: Harness<'static, App>,
    _runtime: Runtime,
    _folder: tempfile::TempDir,
}

impl Fixture {
    pub const fn harness(&mut self) -> &mut Harness<'static, App> {
        &mut self.harness
    }
}

impl HelperFetch for NoHelper {
    fn fetch_command(&self, _path: &ConfigFilePath) -> Command {
        Command::new("false")
    }
}

fn copy_fixture(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_fixture(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

pub fn configs() -> Fixture {
    let runtime = Runtime::new().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let dir = folder.path().join("configs");
    fs::create_dir_all(dir.join("home")).unwrap();
    copy_fixture(&Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE), &dir);
    fs::write(dir.join("home/.bashrc"), "# ~/.bashrc\n").unwrap();

    let setup = ConfigsSetup {
        runtime: runtime.handle().clone(),
        source: Arc::new(SystemConfigFiles::new(
            dir.join("root"),
            dir.join("home"),
            NoHelper,
        )),
        catalog: ConfigFileCatalog::load(&dir).map_err(|error| LoadProblem::from(&error)),
        copies: EditFolder::new(&folder.path().join("copies")),
        editing: Arc::new(DryRunEditor),
        visual: None,
        editor: None,
        dry_run: true,
    };

    let mut harness = window(TALL).build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                configs: Some(setup),
                ..Setup::default()
            },
        )
    });

    harness.run();
    harness
        .get_by_role_and_label(Role::Tab, "Config Files")
        .click();
    step_until(&mut harness, "the list", |harness| {
        harness.query_all_by_label("GRUB default").next().is_some()
    });
    settle(&mut harness);

    Fixture {
        harness,
        _runtime: runtime,
        _folder: folder,
    }
}

#[test]
fn the_files_that_exist_show_by_section() {
    let mut fixture = configs();
    let harness = fixture.harness();

    for label in [
        "Bootloader",
        "System",
        "User environment",
        "Hostname",
        "Bash",
    ] {
        assert!(
            harness.query_all_by_label(label).next().is_some(),
            "{label}"
        );
    }

    assert!(harness.query_all_by_label("Fstab").next().is_some());
}

#[test]
fn a_guarded_file_is_shown_read_only() {
    let mut fixture = configs();
    let harness = fixture.harness();

    harness
        .query_all_by_label("View")
        .next()
        .unwrap()
        .scroll_to_me();
    harness.run();
    harness.query_all_by_label("View").next().unwrap().click();
    step_until(harness, "the text", |harness| {
        harness.query_all_by_label("Copy all").next().is_some()
    });

    assert!(
        harness
            .query_all_by_label_contains("The text of /etc/group")
            .next()
            .is_some()
    );
}
