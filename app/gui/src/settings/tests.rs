use std::path::Path;
use std::time::{Duration, Instant};

use toolbox_config::{Autostart, RefreshInterval, Settings};

use super::{FileStore, SettingsStore as _};
use crate::testing::PATIENCE;

fn eventually(done: impl Fn() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < PATIENCE {
        if done() {
            return true;
        }

        std::thread::sleep(Duration::from_millis(5));
    }

    false
}

fn store(dir: &Path) -> (FileStore, Autostart) {
    let autostart = Autostart::new(dir.join("autostart"), dir.join("garuda-toolbox.desktop"));
    (
        FileStore::new(dir.join("settings.toml"), autostart.clone()),
        autostart,
    )
}

#[test]
fn a_change_reaches_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let (store, _) = store(dir.path());
    let mut settings = Settings::default();
    settings.status.refresh = RefreshInterval::Minutes(45);

    store.save(settings.clone());

    let path = dir.path().join("settings.toml");
    assert!(eventually(|| Settings::load(&path).settings == settings));
}

#[test]
fn start_at_login_sets_and_takes_back_the_link() {
    let dir = tempfile::tempdir().unwrap();
    let (store, autostart) = store(dir.path());

    store.set_autostart(true);
    assert!(eventually(|| autostart.is_enabled()));

    store.set_autostart(false);
    assert!(eventually(|| !autostart.is_enabled()));
}
