use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use toolbox_plan::PackageName;
use toolbox_system::AppIconSource;

use super::{Layered, PictureFinder, Pictures};
use crate::desktop::icon_files::IconLookup;
use crate::testing::PATIENCE;

/// Answers from a table and counts what it was asked.
#[derive(Default)]
struct Table {
    files: HashMap<(String, String), PathBuf>,
    asked: Option<crossbeam_channel::Sender<String>>,
}

impl PictureFinder for Table {
    fn find(&mut self, theme: &str, package: &PackageName) -> Option<PathBuf> {
        if let Some(asked) = &self.asked {
            asked.send(package.to_string()).unwrap();
        }

        self.files
            .get(&(theme.to_owned(), package.to_string()))
            .cloned()
    }
}

struct Theme(&'static [(&'static str, &'static str)]);

impl IconLookup for Theme {
    fn find(&self, _theme: &str, name: &str) -> Option<PathBuf> {
        self.0
            .iter()
            .find(|(icon, _)| *icon == name)
            .map(|(_, file)| PathBuf::from(file))
    }
}

/// Has `kate`'s picture once `installed` holds it, as after a plan.
struct Installs {
    installed: std::sync::Arc<parking_lot::Mutex<Option<PathBuf>>>,
    forgot: crossbeam_channel::Sender<()>,
}

impl PictureFinder for Installs {
    fn find(&mut self, _theme: &str, _package: &PackageName) -> Option<PathBuf> {
        self.installed.lock().clone()
    }

    fn forget(&mut self) {
        self.forgot.send(()).unwrap();
    }
}

struct Cache(&'static [(&'static str, &'static str)]);

impl AppIconSource for Cache {
    fn icon_file(&self, package: &str) -> Option<PathBuf> {
        self.0
            .iter()
            .find(|(name, _)| *name == package)
            .map(|(_, file)| PathBuf::from(file))
    }
}

fn package(name: &str) -> PackageName {
    name.parse().unwrap()
}

fn table(entries: &[(&str, &str, &str)]) -> Table {
    Table {
        files: entries
            .iter()
            .map(|(theme, name, file)| (((*theme).to_owned(), (*name).to_owned()), file.into()))
            .collect(),
        asked: None,
    }
}

/// Asks until the answer is in, as frames would. A lookup that never
/// answers fails the test rather than passing on its empty value.
fn settle(pictures: &Pictures, name: &str) -> Option<PathBuf> {
    let deadline = Instant::now() + PATIENCE;
    let package = package(name);

    loop {
        pictures.file(&package);
        if !pictures.waiting(&package) {
            return pictures.file(&package).map(|file| file.to_path_buf());
        }

        assert!(
            Instant::now() < deadline,
            "{name}: no answer within {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn nothing_is_asked_before_the_theme_is_known() {
    let (finder, asked) = counted(&[("Breeze", "firefox", "/i/firefox.svg")]);
    let pictures = Pictures::spawn(egui::Context::default(), finder);

    assert_eq!(pictures.file(&package("firefox")), None);
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(asked.try_iter().count(), 0);
}

#[test]
fn a_picture_arrives_once_and_is_kept() {
    let (finder, asked) = counted(&[("Breeze", "firefox", "/i/firefox.svg")]);
    let pictures = Pictures::spawn(egui::Context::default(), finder);
    pictures.set_theme("Breeze");

    assert_eq!(settle(&pictures, "firefox"), Some("/i/firefox.svg".into()));
    assert_eq!(settle(&pictures, "aerc"), None);
    pictures.file(&package("firefox"));
    pictures.file(&package("aerc"));

    assert_eq!(asked.try_iter().collect::<Vec<_>>(), ["firefox", "aerc"]);
}

#[test]
fn another_theme_looks_again() {
    let finder = table(&[
        ("Breeze", "firefox", "/breeze/firefox.svg"),
        ("Papirus", "firefox", "/papirus/firefox.svg"),
    ]);

    let pictures = Pictures::spawn(egui::Context::default(), finder);

    pictures.set_theme("Breeze");
    assert_eq!(
        settle(&pictures, "firefox"),
        Some("/breeze/firefox.svg".into())
    );

    pictures.set_theme("Papirus");

    assert_eq!(
        settle(&pictures, "firefox"),
        Some("/papirus/firefox.svg".into())
    );
}

#[test]
fn the_theme_comes_first_under_the_plain_name_then_the_cache() {
    let theme = Theme(&[("brave", "/theme/brave.svg")]);
    let mut finder = Layered::new(theme, || {
        Cache(&[
            ("brave-bin", "/cache/brave.jxl"),
            ("aerc", "/cache/aerc.jxl"),
        ])
    });

    assert_eq!(
        finder.find("Breeze", &package("brave-bin")),
        Some(Path::new("/theme/brave.svg").to_path_buf())
    );
    assert_eq!(
        finder.find("Breeze", &package("aerc")),
        Some(Path::new("/cache/aerc.jxl").to_path_buf())
    );
    assert_eq!(finder.find("Breeze", &package("kate")), None);
}

fn counted(entries: &[(&str, &str, &str)]) -> (Table, crossbeam_channel::Receiver<String>) {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let mut finder = table(entries);
    finder.asked = Some(sender);
    (finder, receiver)
}

#[test]
fn a_miss_is_asked_again_once_a_plan_may_have_installed_the_app() {
    let installed = std::sync::Arc::default();
    let (forgot, forgotten) = crossbeam_channel::unbounded();
    let finder = Installs {
        installed: std::sync::Arc::clone(&installed),
        forgot,
    };

    let pictures = Pictures::spawn(egui::Context::default(), finder);
    pictures.set_theme("Breeze");
    assert_eq!(settle(&pictures, "kate"), None);

    *installed.lock() = Some(PathBuf::from("/i/kate.svg"));
    assert_eq!(
        settle(&pictures, "kate"),
        None,
        "a miss was asked again unasked"
    );

    pictures.forget_misses();
    assert_eq!(settle(&pictures, "kate"), Some("/i/kate.svg".into()));
    assert_eq!(forgotten.try_iter().count(), 1);
}

#[test]
fn the_layers_forget_the_index_and_read_the_cache_again() {
    let reads = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = std::sync::Arc::clone(&reads);
    let mut finder = Layered::new(Theme(&[]), move || {
        counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Cache(&[])
    });

    finder.find("Breeze", &package("kate"));
    finder.find("Breeze", &package("aerc"));
    finder.forget();
    finder.find("Breeze", &package("kate"));

    assert_eq!(reads.load(std::sync::atomic::Ordering::SeqCst), 2);
}
