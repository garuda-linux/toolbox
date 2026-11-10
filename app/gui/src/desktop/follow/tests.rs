#![expect(
    clippy::disallowed_methods,
    reason = "tests compare against literal colours"
)]

use std::ffi::OsString;
use std::fs;
use std::time::Duration;

use egui::{Color32, Theme};
use tempfile::TempDir;
use tokio::sync::watch;
use toolbox_gui_kit::desktop::family::Family;

use super::spawn;
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

    let path = home.path().to_owned();
    let files = DesktopFiles::from_env(move |key| (key == "HOME").then(|| OsString::from(&path)));
    let mut receiver = spawn(
        &tokio::runtime::Handle::current(),
        egui::Context::default(),
        Family::Kde,
        files,
        Vec::new().into(),
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
