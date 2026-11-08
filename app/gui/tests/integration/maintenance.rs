use std::path::Path;
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use toolbox_catalog::SkelCatalog;
use toolbox_gui::catalogs::Catalogs;
use toolbox_gui::maintenance::{MaintenanceSetup, SampleMirrors};
use toolbox_gui::programs::Programs;
use toolbox_gui::{App, Setup, reader};

use crate::common::{TALL, open_tab, palette, window};
use crate::system::{shows, step_until};

const CATALOG: &str = r#"
[[config]]
id = "kitty"
name = "Kitty"
files = ["/etc/skel/.config/kitty/kitty.conf"]

[[config]]
id = "fish"
name = "Fish"
files = ["/etc/skel/.config/fish/config.fish"]
"#;

/// runtime runs the app's jobs, so it lives as long as the harness.
pub struct Fixture {
    harness: Harness<'static, App>,
    home: TempDir,
    _skel: TempDir,
    _runtime: Runtime,
}

impl Fixture {
    pub const fn harness(&mut self) -> &mut Harness<'static, App> {
        &mut self.harness
    }

    pub fn home(&self) -> &Path {
        self.home.path()
    }
}

fn programs() -> Programs {
    Programs::found(|path| path != Path::new("/usr/bin/btrfs-assistant-launcher"))
}

pub fn maintenance() -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let skel = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    for (file, text) in [
        (".config/kitty/kitty.conf", "font_size 12\n"),
        (".config/fish/config.fish", "set fish_greeting\n"),
    ] {
        let path = skel.path().join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    let setup = MaintenanceSetup {
        runtime: handle.clone(),
        mirrors: Arc::new(SampleMirrors),
        home: home.path().to_owned(),
        skel_root: skel.path().to_owned(),
    };

    let mut harness = window(TALL).build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                programs: Some(reader::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    "programs",
                    programs,
                )),
                catalogs: Catalogs {
                    skel: Some(SkelCatalog::parse(CATALOG, "skel.toml").unwrap()),
                    ..Catalogs::default()
                },
                maintenance: Some(setup),
                ..Setup::default()
            },
        )
    });

    harness.run();
    harness
        .get_by_role_and_label(Role::Tab, "Maintenance")
        .click();
    step_until(&mut harness, "the tools", shows("Btrfs Assistant"));
    harness.run();
    Fixture {
        harness,
        home,
        _skel: skel,
        _runtime: runtime,
    }
}

#[test]
fn a_reset_copies_the_chosen_settings_into_the_home() {
    // The fixture stays whole: its runtime runs the reset.
    let mut fixture = maintenance();
    let harness = fixture.harness();

    open_tab(harness, "Reset settings");
    harness
        .get_by_role_and_label(Role::CheckBox, "Kitty")
        .click();
    harness.run();
    harness.get_by_label("Reset selected…").click();
    harness.run();
    harness.get_by_label("Reset").click();
    step_until(
        harness,
        "the reset's end",
        shows("Settings reset. The old files end in .bak and the date."),
    );

    let copied = std::fs::read_to_string(fixture.home().join(".config/kitty/kitty.conf"));
    assert_eq!(copied.unwrap(), "font_size 12\n");
    assert!(!fixture.home().join(".config/fish").exists());
}

#[test]
fn repairs_shows_the_repairs_of_the_installed_programs() {
    let mut fixture = maintenance();
    let harness = fixture.harness();

    open_tab(harness, "Repairs");

    assert!(harness.query_by_label("Reset audio").is_some());
    assert!(harness.query_by_label("Install garuda-update").is_none());
}

fn pick(harness: &mut Harness<'static, App>, text: &str) {
    palette(harness, text);
    harness.key_press(Key::Enter);
    harness.run();
}

#[test]
fn the_palette_opens_the_tab_that_holds_a_repair() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    harness.run();

    pick(harness, "reset audio");

    assert!(harness.query_by_label("Reset audio").is_some());
    assert!(harness.query_by_label("Update system").is_none());
}

#[test]
fn the_palette_finds_an_operation_and_the_reset() {
    let mut fixture = maintenance();
    let harness = fixture.harness();
    open_tab(harness, "Repairs");

    pick(harness, "orphans");
    assert!(harness.query_by_label("Remove orphans").is_some());

    pick(harness, "reset settings");
    assert!(harness.query_by_label("Application settings").is_some());
}
