#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use std::cell::RefCell;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use egui::{Color32, Theme};
use tempfile::TempDir;
use tokio::sync::watch;
use toolbox_gui_kit::desktop::FontSpec;
use toolbox_gui_kit::desktop::family::Family;
use toolbox_gui_kit::fonts::FontFile;
use toolbox_system::PortalSettings;
use toolbox_system::testing::look_portal::FixedLook;

use super::{spawn, ui_faces};
use crate::desktop::DesktopLook;
use crate::desktop::files::DesktopFiles;

const WAIT: Duration = Duration::from_secs(10);

/// `/usr/share/color-schemes/BreezeDark.colors`, whose window is 32,35,38.
const BREEZE_DARK: &str = toolbox_gui_kit::testing::desktop::kde::BREEZEDARK_COLORS;

/// [`BREEZE_DARK`] with its window colour edited to a light grey.
fn light() -> String {
    let window = "[Colors:Window]\nBackgroundAlternate=41,44,48\nBackgroundNormal=32,35,38";
    let light = BREEZE_DARK.replace(
        window,
        "[Colors:Window]\nBackgroundAlternate=41,44,48\nBackgroundNormal=240,240,240",
    );

    assert_ne!(light, BREEZE_DARK);
    light
}

fn files_in(home: &Path) -> DesktopFiles {
    let path = home.to_owned();
    DesktopFiles::from_env(move |key| match key {
        "HOME" => Some(OsString::from(&path)),
        "XDG_CONFIG_DIRS" => Some(path.join("etc-xdg").into_os_string()),
        _ => None,
    })
}

async fn next_look(receiver: &mut watch::Receiver<DesktopLook>) -> DesktopLook {
    tokio::time::timeout(WAIT, receiver.changed())
        .await
        .expect("a look arrives in time")
        .expect("the task is still running");
    receiver.borrow_and_update().clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_look_follows_an_edited_kdeglobals() {
    let home = TempDir::new().unwrap();
    let config = home.path().join(".config");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("kdeglobals"), light()).unwrap();

    let mut receiver = spawn(
        &tokio::runtime::Handle::current(),
        egui::Context::default(),
        Family::Kde,
        (files_in(home.path()), Arc::new(FixedLook::default())),
        super::CatalogApps {
            names: Vec::new().into(),
            pictures: crate::desktop::icon_files::app_index(Vec::new()),
        },
    );

    let first = next_look(&mut receiver).await;
    assert_eq!(first.look.theme, Theme::Light);
    assert_eq!(first.look.light.window, Color32::from_rgb(240, 240, 240));

    // A desktop writes a new file and renames it over the old one.
    fs::write(config.join("kdeglobals.new"), BREEZE_DARK).unwrap();
    fs::rename(config.join("kdeglobals.new"), config.join("kdeglobals")).unwrap();

    let second = next_look(&mut receiver).await;
    assert_eq!(second.look.theme, Theme::Dark);
    assert_eq!(second.look.dark.window, Color32::from_rgb(32, 35, 38));
}

/// Without a desktop file naming a theme, the portal's preference decides it.
#[tokio::test(flavor = "multi_thread")]
async fn the_portal_preference_sets_the_theme_without_desktop_files() {
    let home = TempDir::new().unwrap();
    let portal = FixedLook(PortalSettings {
        dark: Some(false),
        ..PortalSettings::default()
    });

    let mut receiver = spawn(
        &tokio::runtime::Handle::current(),
        egui::Context::default(),
        Family::Kde,
        (files_in(home.path()), Arc::new(portal)),
        super::CatalogApps {
            names: Vec::new().into(),
            pictures: crate::desktop::icon_files::app_index(Vec::new()),
        },
    );

    let look = next_look(&mut receiver).await;
    assert_eq!(look.look.theme, Theme::Light);
}

#[test]
fn the_regular_face_is_the_desktop_weight_and_bold_asks_for_700() {
    let spec = FontSpec {
        family: "Inter".to_owned(),
        points: 10.0,
        weight: 700,
    };

    let asked = RefCell::new(Vec::new());
    let faces = ui_faces(&spec, |wanted| {
        asked.borrow_mut().push(wanted.weight);
        Some(FontFile {
            name: wanted.family,
            data: Vec::new(),
            index: 0,
            weight: wanted.weight,
        })
    });

    assert_eq!(asked.into_inner(), [700, 700]);
    assert_eq!(faces.regular.map(|face| face.weight), Some(700));
    assert_eq!(faces.bold.map(|face| face.weight), Some(700));
}

#[test]
fn a_desktop_at_regular_weight_keeps_bold_at_700() {
    let spec = FontSpec {
        family: "Noto Sans".to_owned(),
        points: 10.0,
        weight: 400,
    };

    let asked = RefCell::new(Vec::new());
    ui_faces(&spec, |wanted| {
        asked.borrow_mut().push(wanted.weight);
        None
    });

    assert_eq!(asked.into_inner(), [400, 700]);
}
