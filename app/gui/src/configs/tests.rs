use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Runtime;
use toolbox_catalog::ConfigFileCatalog;
use toolbox_core::features::configs::session::Phase;
use toolbox_core::features::configs::{ConfigEntrySpec, Found, Listing, viewing};
use toolbox_exec::ConfigRead;
use toolbox_plan::{
    ConfigFilePath, ConfigText, Editor, HomePath, PlannedAction, RootAction, UserAction,
};
use toolbox_system::{ConfigFileSource, DryRunEditor, EditFolder, ReadProblem};

use super::{Act, Configs, ConfigsSetup, Taken};
use crate::testing::until;

const CATALOG: &str = r#"
[[section]]
id = "boot"
title = "Bootloader"
icon = "system-reboot"

[[section.entry]]
id = "grub"
name = "GRUB default"
path = "/etc/default/grub"

[[section]]
id = "users"
title = "Users"
icon = "system-users"

[[section.entry]]
id = "passwd"
name = "passwd"
path = "/etc/passwd"
read_only = true

[[section]]
id = "shell"
title = "Shell"
icon = "utilities-terminal"

[[section.entry]]
id = "fish"
name = "fish"
path = "~/.config/fish/config.fish"
"#;

/// A machine with GRUB's file and `/etc/passwd`. Counts the lists. With
/// `hold` a read waits until it is cancelled, as one behind the password
/// dialog does.
#[derive(Default)]
struct Machine {
    lists: AtomicUsize,
    hold: bool,
    waiting: AtomicBool,
    cancelled: AtomicBool,
    home: Option<PathBuf>,
}

impl ConfigFileSource for Machine {
    fn listing(&self, _entries: &[ConfigEntrySpec]) -> Listing {
        self.lists.fetch_add(1, Ordering::SeqCst);
        let mut listing = Listing::default();
        listing.insert("grub", Found::File { link: None });
        listing.insert("passwd", Found::File { link: None });
        listing.insert("fish", Found::File { link: None });
        listing
    }

    fn read(
        &self,
        path: &ConfigFilePath,
        cancel: &Arc<AtomicBool>,
    ) -> Result<ConfigRead, ReadProblem> {
        if self.hold {
            self.waiting.store(true, Ordering::SeqCst);
            until("the read was never cancelled", || {
                cancel.load(Ordering::SeqCst)
            });
            self.cancelled.store(true, Ordering::SeqCst);
            return Err(ReadProblem::Cancelled);
        }

        Ok(ConfigRead {
            path: path.clone(),
            content: ConfigText::new("GRUB_TIMEOUT=5\n").unwrap(),
        })
    }

    fn installed(&self, _editor: Editor) -> bool {
        true
    }

    fn home_file(&self, path: &HomePath) -> Option<PathBuf> {
        self.home.as_ref().map(|home| home.join(path.as_str()))
    }
}

fn configs() -> (Runtime, Arc<Machine>, tempfile::TempDir, Configs) {
    configs_on(Machine::default())
}

fn configs_on(machine: Machine) -> (Runtime, Arc<Machine>, tempfile::TempDir, Configs) {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(machine);
    let folder = tempfile::tempdir().unwrap();
    let setup = ConfigsSetup {
        runtime: runtime.handle().clone(),
        source: Arc::<Machine>::clone(&machine),
        catalog: Ok(ConfigFileCatalog::parse(CATALOG, "test.toml").unwrap()),
        copies: EditFolder::new(folder.path()),
        editing: Arc::new(DryRunEditor),
        visual: None,
        editor: None,
        dry_run: true,
    };

    let configs = Configs::new(Some(setup), egui::Context::default());
    (runtime, machine, folder, configs)
}

fn settle(configs: &mut Configs, done: impl Fn(&Configs, &Taken) -> bool) -> Taken {
    let mut all = Taken::default();
    until("the results never came", || {
        let taken = configs.take();
        all.acts.extend(taken.acts);
        all.review |= taken.review;
        done(configs, &all)
    });
    all
}

fn only(act: &Act) -> Option<&PlannedAction> {
    match act {
        Act::Run(plan) => match plan.steps() {
            [step] => Some(&step.action),
            _ => None,
        },
        Act::Cancel => None,
    }
}

fn is_editor(act: &Act) -> bool {
    matches!(
        only(act),
        Some(PlannedAction::User(UserAction::EditFile {
            editor: _,
            copy: _
        }))
    )
}

#[test]
fn the_files_are_listed_once_the_page_opens() {
    let (_runtime, machine, _folder, mut configs) = configs();

    assert_eq!(machine.lists.load(Ordering::SeqCst), 0);

    configs.open();
    settle(&mut configs, |configs, _| configs.listing().is_some());
    configs.open();

    // A second listing would run on a blocking thread, so it gets the time
    // to land before the count.
    std::thread::sleep(Duration::from_millis(50));
    configs.take();

    assert_eq!(machine.lists.load(Ordering::SeqCst), 1);
}

#[test]
fn a_page_opened_every_frame_still_gets_its_one_listing() {
    let (_runtime, machine, _folder, mut configs) = configs();

    until("the listing never landed", || {
        configs.open();
        configs.take();
        configs.listing().is_some()
    });

    assert_eq!(machine.lists.load(Ordering::SeqCst), 1);
}

#[test]
fn an_edit_runs_the_editor_and_writes_after_the_review() {
    let (_runtime, _machine, _folder, mut configs) = configs();
    configs.open();
    settle(&mut configs, |configs, _| configs.listing().is_some());

    assert_eq!(configs.edit("grub"), Ok(None));

    let edited = settle(&mut configs, |_, taken| taken.acts.iter().any(is_editor));
    assert!(!edited.review);

    assert_eq!(configs.plan_started(), None);
    assert_eq!(configs.plan_finished(true), None);

    let reviewed = settle(&mut configs, |_, taken| taken.review);
    assert_eq!(
        configs.session().phase(),
        Phase::Review,
        "{:?}",
        configs.session().problem()
    );
    assert_eq!(reviewed.acts, []);

    let write = configs.apply().unwrap();
    assert!(matches!(
        only(&write),
        Some(PlannedAction::Root(RootAction::WriteConfigFile {
            write: _
        }))
    ));
}

#[test]
fn a_read_only_file_is_viewed_and_never_edited() {
    let (_runtime, _machine, _folder, mut configs) = configs();
    configs.open();
    settle(&mut configs, |configs, _| configs.listing().is_some());

    assert_eq!(configs.edit("passwd"), Err("read-only"));
    assert_eq!(configs.view("passwd"), Ok(()));
    assert_eq!(configs.view("passwd"), Err("busy"));

    settle(&mut configs, |configs, _| {
        configs.viewing().phase() == viewing::Phase::Shown
    });

    configs.close_view();

    assert_eq!(configs.viewing().phase(), viewing::Phase::Idle);
    assert_eq!(configs.edit("nothing"), Err("unknown"));
}

#[test]
fn a_read_behind_the_password_dialog_ends_with_the_page() {
    let (_runtime, machine, _folder, mut configs) = configs_on(Machine {
        hold: true,
        ..Machine::default()
    });

    configs.open();
    settle(&mut configs, |configs, _| configs.listing().is_some());

    assert_eq!(configs.view("passwd"), Ok(()));
    until("the read never started", || {
        machine.waiting.load(Ordering::SeqCst)
    });
    drop(configs);

    until("the read never ended", || {
        machine.cancelled.load(Ordering::SeqCst)
    });
}

#[test]
fn a_file_of_the_user_is_edited_as_a_copy_and_written_as_the_user() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join(".config/fish")).unwrap();
    std::fs::write(home.path().join(".config/fish/config.fish"), "set a 1\n").unwrap();
    let (_runtime, _machine, _folder, mut configs) = configs_on(Machine {
        home: Some(home.path().to_path_buf()),
        ..Machine::default()
    });

    configs.open();
    settle(&mut configs, |configs, _| configs.listing().is_some());

    assert_eq!(configs.edit("fish"), Ok(None));
    settle(&mut configs, |_, taken| taken.acts.iter().any(is_editor));
    configs.plan_started();
    configs.plan_finished(true);
    settle(&mut configs, |_, taken| taken.review);

    let write = configs.apply().unwrap();
    assert!(
        matches!(
            only(&write),
            Some(PlannedAction::User(UserAction::WriteHomeFile { write: _ }))
        ),
        "{write:?}"
    );
}
