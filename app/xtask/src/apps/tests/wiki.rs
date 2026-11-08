use std::collections::BTreeSet;

use proptest::prelude::*;
use toolbox_plan::PackageName;
use toolbox_state::parts::SyncPackages;
use toolbox_system::{ProbeError, SyncSource};

use super::super::App;
use super::super::tasks::{WikiOptions, offered_packages, parse_options};
use super::super::wiki::{WikiApp, keep, parse_app, split_fields};
use super::args;

const FIREFOX: &str = "* {{App|[[Firefox]]|Extensible browser from Mozilla based on Gecko with fast rendering.|https://www.firefox.com/en-US/|{{Pkg|firefox}}}}";
const PKEXEC: &str = "* {{App|{{man|1|pkexec}}|A [[Polkit]] application that allows an authorized user to run commands or an interactive shell as another user. Configured using Polkit rules.|https://gitlab.freedesktop.org/polkit/polkit/|{{Pkg|polkit}}}}";
const AQUARIA: &str = "* {{App|[[Wikipedia:Aquaria (video game)|Aquaria]]|2D sidescrolling action-adventure game, heavily focused on exploration and puzzle-solving, with non-linear gameplay. The game engine is open source.|https://www.bit-blot.com/aquaria|{{AUR|aquaria-ose}}, {{AUR|aquaria-hib}}}}";
const MONERO: &str =
    "* {{App|Monero|Monero wallet.|https://getmonero.org/|{{Pkg|monero-gui}}, {{Pkg|monero}}}}";
const DASH: &str = "* {{App|Dash to Panel|A fully customizable icon taskbar for {{Pkg|gnome-shell}}. Moves the dash into the GNOME main panel, similar to KDE Plasma and Windows 10.|https://github.com/home-sweet-gnome/dash-to-panel|{{Pkg|gnome-shell-extension-dash-to-panel}}}}";
const NO_PACKAGE: &str = "* {{App|Neomacs (NEO Emacs)|A GPU-powered Emacs written in Rust with a modern display engine.|https://github.com/eval-exec/neomacs}}";
const DEAD_LINK: &str = "* {{App|Grand Theft Auto III|An open-source project reverse-engineering Grand Theft Auto III.|https://github.com/halpz/re3{{Dead link|2025|08|15|status=451}}|{{AUR|re3-git}}}}";
const ELVIS: &str = "* {{App|Elvis|A ''vi'' clone with optional X support.|http://elvis.the-little-red-haired-girl.org/|{{Aur|elvis}}}}";

fn three() -> Vec<WikiApp> {
    vec![
        wiki("Zed", "zed", false),
        wiki("aerc", "aerc", false),
        wiki("Aur thing", "aur-thing", true),
    ]
}

fn titles(apps: &[App]) -> Vec<&str> {
    apps.iter().map(|app| app.title.as_str()).collect()
}

fn wiki(title: &str, package: &str, aur_only: bool) -> WikiApp {
    WikiApp {
        app: App {
            title: title.to_owned(),
            description: "D".to_owned(),
            url: None,
            packages: vec![package.to_owned()],
        },
        aur_only,
    }
}

#[test]
fn an_app_line_has_title_description_link_and_package() {
    let firefox = parse_app(FIREFOX).unwrap();

    assert_eq!(
        firefox,
        WikiApp {
            app: App {
                title: "Firefox".to_owned(),
                description: "Extensible browser from Mozilla based on Gecko with fast rendering."
                    .to_owned(),
                url: Some("https://www.firefox.com/en-US/".to_owned()),
                packages: vec!["firefox".to_owned()],
            },
            aur_only: false,
        }
    );
}

#[test]
fn templates_and_links_inside_fields_stay_in_their_field() {
    let pkexec = parse_app(PKEXEC).unwrap().app;

    assert_eq!(pkexec.title, "pkexec");
    assert!(pkexec.description.starts_with("A Polkit application"));
    assert_eq!(pkexec.packages, ["polkit"]);

    let aquaria = parse_app(AQUARIA).unwrap();

    assert_eq!(aquaria.app.title, "Aquaria");
    assert_eq!(aquaria.app.packages, ["aquaria-ose", "aquaria-hib"]);
    assert!(aquaria.aur_only);

    let dash = parse_app(DASH).unwrap().app;

    assert!(dash.description.contains("taskbar for gnome-shell. Moves"));
    assert_eq!(dash.packages, ["gnome-shell-extension-dash-to-panel"]);
}

#[test]
fn every_package_of_a_line_is_kept() {
    assert_eq!(
        parse_app(MONERO).unwrap().app.packages,
        ["monero-gui", "monero"]
    );
}

#[test]
fn lines_without_app_or_package_are_skipped() {
    assert_eq!(parse_app(NO_PACKAGE), None);
    assert_eq!(parse_app("== Internet =="), None);
    assert_eq!(parse_app("* {{App|Only|two}}"), None);
}

#[test]
fn dead_links_and_http_sites_lose_their_link() {
    let gta = parse_app(DEAD_LINK).unwrap().app;

    assert_eq!(gta.url.as_deref(), Some("https://github.com/halpz/re3"));
    assert_eq!(gta.packages, ["re3-git"]);

    let elvis = parse_app(ELVIS).unwrap();

    assert_eq!(elvis.app.url, None);
    assert_eq!(elvis.app.description, "A vi clone with optional X support.");
    assert_eq!(elvis.app.packages, ["elvis"], "template names ignore case");
    assert!(elvis.aur_only);
}

#[test]
fn fields_split_outside_nested_templates_and_links() {
    assert_eq!(
        split_fields("[[A|B]]|{{x|y}}|c"),
        ["[[A|B]]", "{{x|y}}", "c"]
    );
    assert_eq!(split_fields("one"), ["one"]);
}

#[test]
fn a_stray_closer_hides_no_later_field() {
    assert_eq!(split_fields("a]b|c"), ["a]b", "c"]);
    assert_eq!(split_fields("]]|x"), ["]]", "x"]);
    assert_eq!(split_fields("{{a]|b}}|c"), ["{{a]|b}}", "c"]);
}

#[test]
fn the_sync_databases_decide_what_stays() {
    let offered: BTreeSet<PackageName> = [PackageName::new("aerc").unwrap()].into();
    let (kept, skipped) = keep(three(), Some(&offered), false);

    assert_eq!(titles(&kept), ["aerc"]);
    assert_eq!(skipped, 2);
}

#[test]
fn without_sync_databases_only_the_aur_goes() {
    let (kept, skipped) = keep(three(), None, false);

    assert_eq!(titles(&kept), ["aerc", "Zed"]);
    assert_eq!(skipped, 1);
}

#[test]
fn an_entry_stays_only_when_every_package_is_offered() {
    let both: BTreeSet<PackageName> = ["monero-gui", "monero"]
        .into_iter()
        .map(|name| PackageName::new(name).unwrap())
        .collect();

    let apps = vec![WikiApp {
        app: App {
            title: "Monero".to_owned(),
            description: "D".to_owned(),
            url: None,
            packages: vec!["monero-gui".to_owned(), "monero".to_owned()],
        },
        aur_only: false,
    }];

    let (kept, skipped) = keep(apps.clone(), Some(&both), false);

    assert_eq!(titles(&kept), ["Monero"]);
    assert_eq!(skipped, 0);

    let first_only: BTreeSet<PackageName> =
        std::iter::once(PackageName::new("monero-gui").unwrap()).collect();

    let (rest, left_out) = keep(apps, Some(&first_only), false);

    assert!(rest.is_empty(), "the unoffered companion was not checked");
    assert_eq!(left_out, 1);
}

#[test]
fn without_sync_databases_the_sync_fails_unless_unfiltered() {
    struct NoDatabases;

    impl SyncSource for NoDatabases {
        fn offered(&self) -> Result<SyncPackages, ProbeError> {
            Err(ProbeError::Unavailable)
        }
    }

    let mut missing = String::new();
    let reason = offered_packages(&NoDatabases, &mut missing, false).unwrap_err();

    assert!(reason.contains("--unfiltered"), "{reason}");
    assert!(missing.is_empty(), "{missing}");

    let mut report = String::new();

    assert_eq!(offered_packages(&NoDatabases, &mut report, true), Ok(None));
    assert!(report.starts_with("warning: "), "{report}");
}

#[test]
fn with_sync_databases_their_packages_are_offered() {
    struct Offers;

    impl SyncSource for Offers {
        fn offered(&self) -> Result<SyncPackages, ProbeError> {
            Ok(SyncPackages(BTreeSet::from([
                PackageName::new("zed").unwrap()
            ])))
        }
    }

    let mut report = String::new();
    let offered = offered_packages(&Offers, &mut report, false)
        .unwrap()
        .unwrap();

    assert_eq!(offered.len(), 1);
    assert!(report.is_empty(), "{report}");
}

#[test]
fn include_aur_keeps_everything() {
    let (kept, skipped) = keep(three(), None, true);

    assert_eq!(kept.len(), 3);
    assert_eq!(skipped, 0);
}

#[test]
fn apps_sort_by_title_with_case_last() {
    let (kept, _) = keep(
        vec![
            wiki("FFmpeg", "ffmpeg", false),
            wiki("Dino", "dino", false),
            wiki("ffmpeg", "ffmpeg", false),
            wiki("Déjà Dup", "deja-dup", false),
        ],
        None,
        false,
    );

    assert_eq!(titles(&kept), ["Déjà Dup", "Dino", "ffmpeg", "FFmpeg"]);
}

#[test]
fn options_parse_every_flag() {
    assert_eq!(parse_options(&[]), Some(WikiOptions::default()));
    assert_eq!(
        parse_options(&args(&["--local", "src", "--include-aur", "--out", "o"])),
        Some(WikiOptions {
            local: Some("src".into()),
            sources: None,
            out: Some("o".into()),
            include_aur: true,
            unfiltered: false,
            write: false,
        })
    );
    assert_eq!(
        parse_options(&args(&["--unfiltered"])),
        Some(WikiOptions {
            unfiltered: true,
            ..WikiOptions::default()
        })
    );
    assert_eq!(parse_options(&args(&["--gaming"])), None);
    assert_eq!(parse_options(&args(&["--local"])), None);
}

proptest! {
    #[test]
    fn splitting_loses_no_character(content in "[ -~]*") {
        prop_assert_eq!(split_fields(&content).join("|"), content);
    }
}

#[test]
fn a_line_with_templates_but_no_package_is_skipped() {
    assert_eq!(
        parse_app("* {{App|Tool|A tool.|https://example.com/|{{Man|1|tool}}}"),
        None
    );
}

#[test]
fn accented_titles_sort_by_their_base_letter() {
    let (kept, _) = keep(
        vec![
            wiki("Zed", "zed", false),
            wiki("ñu", "nu", false),
            wiki("nu", "nu", false),
            wiki("Bar", "bar", false),
            wiki("ça", "ca", false),
            wiki("über", "uber", false),
            wiki("ýla", "yla", false),
        ],
        None,
        false,
    );

    assert_eq!(
        titles(&kept),
        ["Bar", "ça", "nu", "ñu", "über", "ýla", "Zed"]
    );
}
